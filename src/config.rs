use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

pub const DEFAULT_CONFIG_FILE: &str = r#"// please cli configuration
// defaults are listed below
{
    // Your API key for the endpoint (required)
    "api-key": "your_api_key_here",

    // Model to use
    "model": "anthropic/claude-haiku-4.5",

    // Whether to suppress informational messages
    "quiet": false,

    // Shell to use for executing commands
    "shell": "/usr/bin/env sh",

    // Endpoint URL
    "endpoint": "https://openrouter.ai/api/v1",

    // Response format of the model
    // accepted values are "harmony" | "json_schema"
    // if not specified, defaults to "json_schema"
    // "response-format": "json_schema"

    "prompts": {
        // Prompt template for generating shell commands
        "command": "You are an expert in the Linux shell. The user would like to perform a task in the shell. \
 Please return ONLY a single shell command compatible with the user's shell (it will be ran with `$SHELL`). \
 Prefer single-line solutions. Do not include any markdown formatting, explanations, or multiple options. \
 Your answer should just be the raw command that can be executed directly. \
 Do not include $SHELL at the start of the command the user will take care of inserting that. \
 The command should be broken into segments (e.g `echo foo` -> [\"echo\", \"foo\"]). \
 Respond with a JSON object as follows { \"command\":  [\"YOUR\", \"COMMAND\"] }",
    }
}

/* -*- mode: json5 -*- */
/* vim: set ft=json5: */
"#;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Harmony,
    JsonSchema
}
impl Default for ResponseFormat {
    fn default() -> Self {
        ResponseFormat::JsonSchema
    }
} 

impl TryFrom::<&String> for ResponseFormat {
    type Error = String;
    fn try_from(value: &String) -> anyhow::Result<Self, Self::Error> {
        match value.as_str() {
            "json_schema" => Ok(Self::JsonSchema),
            "harmony" => Ok(ResponseFormat::Harmony),
            _ => Err(format!("Unrecognized response format specified {}", value))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, rename = "api-key")]
    pub api_key: String,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default)]
    pub quiet: bool,

    #[serde(default = "default_shell")]
    pub shell: String,

    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    #[serde(default, rename = "response-format")]
    pub response_format: ResponseFormat,

    #[serde(default)]
    pub prompts: Prompts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompts {
    #[serde(default = "default_command_prompt")]
    pub command: String,
}

impl Default for Prompts {
    fn default() -> Self {
        Self {
            command: default_command_prompt(),
        }
    }
}

fn default_model() -> String {
    "anthropic/claude-haiku-4.5".to_string()
}

fn default_shell() -> String {
    "/usr/bin/env sh".to_string()
}

fn default_endpoint() -> String {
    "https://openrouter.ai/api/v1".to_string()
}

fn default_response_format() -> ResponseFormat {
    ResponseFormat::JsonSchema
}

fn default_command_prompt() -> String {
    r#"You are an expert in the Linux shell. The user would like to perform a task in the shell.
Please return ONLY a single shell command compatible with the user's shell (it will be ran with `$SHELL`).
Prefer single-line solutions. Do not include any markdown formatting, explanations, or multiple options.
Your answer should just be the raw command that can be executed directly.
Do not include $SHELL at the start of the command the user will take care of inserting that.
The command should be broken into segments (e.g `echo foo` -> ["echo", "foo"]).
Respond with a JSON object as follows { "command":  ["YOUR", "COMMAND"] }"#.to_string()
}

impl Config {
    /// Load configuration from XDG config directory and environment variables
    pub fn load() -> Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("please")
            .context("Failed to initialize XDG directories")?;

        // Try to load config.json5 first, then config.json
        let config_path = xdg_dirs
            .find_config_file("config.json5")
            .or_else(|| xdg_dirs.find_config_file("config.json"));

        let mut config = if let Some(path) = config_path {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;

            json5::from_str::<Config>(&content)
                .with_context(|| format!("Failed to parse config file: {}", path.display()))?
        } else {
            // No config file found, use defaults
            Config {
                api_key: String::new(),
                model: default_model(),
                quiet: false,
                shell: default_shell(),
                endpoint: default_endpoint(),
                response_format: default_response_format(),
                prompts: Prompts::default(),
            }
        };

        // Override with environment variables
        if let Ok(api_key) = env::var("PLEASE_API_KEY") {
            config.api_key = api_key;
        }

        if let Ok(model) = env::var("PLEASE_MODEL") {
            config.model = model;
        }

        if let Ok(shell) = env::var("PLEASE_SHELL") {
            config.shell = shell;
        }

        if let Ok(endpoint) = env::var("PLEASE_ENDPOINT") {
            config.endpoint = endpoint;
        }

        if let Ok(response_format) = env::var("PLEASE_RESPONSE_FORMAT") {
            if let Ok(rf) = ResponseFormat::try_from(&response_format) {
                config.response_format = rf;
            } else {
                let msg = format!("Unrecognized response format specified in PLEASE_RESPONSE_FORMAT ({})", response_format);
                anyhow::bail!(msg);
            }
        }

        if let Ok(quiet) = env::var("PLEASE_QUIET") {
            config.quiet = quiet == "1" || quiet.to_lowercase() == "true";
        }

        if let Ok(command_prompt) = env::var("PLEASE_PROMPTS_COMMAND") {
            config.prompts.command = command_prompt;
        }
        Ok(config)
    }

    /// Get the command prompt with variables substituted
    pub fn get_command_prompt(&self) -> String {
        self.prompts.command.replace("$SHELL", &self.shell)
    }
}
