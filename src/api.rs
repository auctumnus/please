use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::{Config, ResponseFormat};

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct CommandResponse {
    command: Vec<String>,
}

pub struct ApiClient {
    client: reqwest::Client,
    endpoint: String,
    api_key: String,
    model: String,
}

impl ApiClient {
    pub fn new(endpoint: String, api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint,
            api_key,
            model,
        }
    }

    /// Request a shell command from the LLM
    pub async fn request_command(
        &self,
        system_prompt: &str,
        user_message: &str,
        config: &Config,
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.endpoint);

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_message.to_string(),
            },
        ];

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "response_format": {
                "type": "json_schema",
                "name": "command_response",
                "strict": true,
                "schema": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute"
                        }
                    },
                    "required": ["command"]
                }
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;

        let raw_response = chat_response
            .choices
            .first().map(|choice| choice.message.content.trim().to_string())
            .context("No response from API")?;

        // Clean up the response (remove markdown code blocks)
        let command = match config.response_format {
            ResponseFormat::Harmony => cleave_start_parse_json(&raw_response),
            _ => Ok(clean_command_response(&raw_response)),
        };
        command
    }

    /// Continue a conversation with feedback from the user
    pub async fn refine_command(
        &self,
        system_prompt: &str,
        original_request: &str,
        previous_command: &str,
        feedback: &str,
        config: &Config,
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.endpoint);

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: original_request.to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: previous_command.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: feedback.to_string(),
            },
        ];

        let request_body = ChatRequest {
            model: self.model.clone(),
            messages,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;

        let raw_response = chat_response
            .choices
            .first().map(|choice| choice.message.content.trim().to_string())
            .context("No response from API")?;

        // Clean up the response (remove markdown code blocks)
        let command = match config.response_format {
            ResponseFormat::Harmony => cleave_start_parse_json(&raw_response),
            _ => Ok(clean_command_response(&raw_response)),
        };
        command
    }
}

fn cleave_start_parse_json(response: &str) -> Result<String> {
    let regex = Regex::new(r"(?m)<\|end\|>(\{.*\}$)").unwrap();
    let captures = regex
        .captures(response)
        .context("Harmony parse - Failed to match regex")?;

    let json_str = captures
        .get(1)
        .context("Harmony parse - Empty json section")?;

    let json_str = json_str.as_str();
    let command_response: CommandResponse = serde_json::from_str(json_str)
        .expect(format!("Failed to parse JSON: {}", json_str).as_str());
    return Ok(command_response.command.join(" "));
}

/// Clean up the command response by removing markdown code blocks and extra text
fn clean_command_response(response: &str) -> String {
    let response = response.trim();

    // Check if the response contains markdown code blocks
    if response.contains("```") {
        // Extract just the first code block
        if let Some(start) = response.find("```") {
            let after_start = &response[start + 3..];

            // Skip the language identifier if present (e.g., "sh\n" or "bash\n")
            let code_start = if let Some(newline_pos) = after_start.find('\n') {
                newline_pos + 1
            } else {
                0
            };

            let code_section = &after_start[code_start..];

            if let Some(end) = code_section.find("```") {
                return code_section[..end].trim().to_string();
            }
        }
    }

    // If no code block found, return the first line or the whole response if it's short
    let lines: Vec<&str> = response.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() == 1 {
        lines[0].trim().to_string()
    } else {
        // Take the first non-empty line if it looks like a command
        lines.first().map(|l| l.trim().to_string()).unwrap_or_else(|| response.to_string())
    }
}
