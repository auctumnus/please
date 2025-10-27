mod api;
mod config;
mod ui;

use anyhow::{Context, Result};
use api::ApiClient;
use config::Config;
use std::{env};
use std::process::Command;
use ui::{UserAction, UI};
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<()> {
    // Check if user is asking for a command directly
    let args: Vec<String> = env::args().collect();

    // Load configuration
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Create API client
    let api_client = ApiClient::new(
        config.endpoint.clone(),
        config.api_key.clone(),
        config.model.clone(),
    );

    // Create UI
    let mut ui = UI::new(config.clone())?;

    // Get the user's request
    let user_request =  if let Some(command) = args.get(1) && args.len() == 2 {
        match command.as_str() {
            "--help" | "-h" | "help" => {
                help();
                return Ok(());
            }
            "--config" | "-C" | "config" => {
                open_config(&mut ui)?;
                return Ok(());
            }
            "--continue" | "-c" | "continue" => {
                r#continue(&mut ui)?;
                return Ok(());
            }
            "die" | "exit" | "quit" => {
                ui.show_message("not very nice...");
                return Ok(());
            }
            _ => command.to_owned(),
        }
    } else if args.len() > 1 {
        // Command line arguments provided
        args[1..].join(" ")
    } else {
        help();
        return Ok(());
    };

    // Get the system prompt with variables substituted
    let system_prompt = config.get_command_prompt();

    // Validate that API key is set
    if config.api_key.is_empty() {
        anyhow::bail!(
            "API key not found. Please set it in the config file or via PLEASE_API_KEY environment variable.\n\
                Expected config location: ~/.config/please/config.json5"
        );
    }

    // Request initial command from API
    if !config.quiet {
        ui.show_message("Thinking...");
    }
    let mut current_command = match api_client
        .request_command(&system_prompt, &user_request)
        .await
    {
        Ok(cmd) => cmd,
        Err(e) => {
            ui.show_error(&format!("Failed to get command: {}", e));
            std::process::exit(1);
        }
    };

    // Main interaction loop
    loop {
        match ui.display_command_and_get_action(&current_command)? {
            UserAction::RunCommand(cmd) => {
                // Execute the command
                run_command(&cmd, &config.shell)?;
                break;
            }
            UserAction::EditCommand(edited_cmd) => {
                // User manually edited the command, run it
                run_command(&edited_cmd, &config.shell)?;
                break;
            }
            UserAction::ProvideFeedback(feedback) => {
                // User provided feedback, refine the command
                ui.show_message("Refining...");
                match api_client
                    .refine_command(
                        &system_prompt,
                        &user_request,
                        &current_command,
                        &feedback,
                    )
                    .await
                {
                    Ok(new_cmd) => {
                        current_command = new_cmd;
                    }
                    Err(e) => {
                        ui.show_error(&format!("Failed to refine command: {}", e));
                        break;
                    }
                }
            }
            UserAction::Quit => {
                break;
            }
        }
    }

    Ok(())
}

fn help() {
    // follow http://docopt.org/
    println!(r#"Usage:
    please <request>...
    please help | -h | --help
    please continue | -c | --continue
    please config | -C | --config

Options:
    -h --help       Show this help message.
    -c --continue   Continue the last session.
    -C --config     Open the configuration file in the default editor ($EDITOR).

Examples:
    please find all .rs files modified in the last 2 days
    please search for 'TODO' in all .py files and count occurrences
    please list all running Docker containers"#);
}

fn open_config(ui: &mut UI) -> Result<()> {
    let Ok(editor) = env::var("EDITOR") else {
        ui.show_error(r#"EDITOR environment variable not set.
To edit the config yourself, go to $XDG_CONFIG_HOME/please/config.json5 or $HOME/.config/please/config.json5"#);
        std::process::exit(1);
    };
    let xdg_dirs = xdg::BaseDirectories::with_prefix("please")
        .context("Failed to initialize XDG directories")?;

    // Try to load config.json5 first, then config.json
    let config_path = xdg_dirs
        .find_config_file("config.json5")
        .or_else(|| xdg_dirs.find_config_file("config.json"));

    if let Some(path) = config_path {
        Command::new(editor)
            .arg(path)
            .status()
            .context("Failed to open config file in editor")?;
        Ok(())
    } else if ui.show_prompt(format!("No config file found. Create one now? {}: ", "(y/n)".bright_black()))? == "y" {
        let config_dir = xdg_dirs
            .get_config_home();
        std::fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        let config_file_path = config_dir.join("config.json5");
        std::fs::write(&config_file_path, config::DEFAULT_CONFIG_FILE)
            .context("Failed to write default config file")?;
        Command::new(editor)
            .arg(config_file_path)
            .status()
            .context("Failed to open config file in editor")?;
        Ok(())
    } else {
        Ok(())
    }
}

fn run_command(command: &str, shell: &str) -> Result<()> {
    // Parse the shell command (e.g., "/usr/bin/env sh" -> ["/usr/bin/env", "sh"])
    let shell_parts: Vec<&str> = shell.split_whitespace().collect();

    if shell_parts.is_empty() {
        anyhow::bail!("Invalid shell configuration");
    }

    let shell_name = shell_parts.last().unwrap();

    // Only add shopt for bash and zsh (which support it)
    // Fish, sh, and other shells don't support shopt
    let command = if shell_name.contains("bash") || shell_name.contains("zsh") {
        format!("shopt -s extglob globstar nullglob\n{command}")
    } else {
        command.to_string()
    };

    let status = if shell_parts.len() == 1 {
        Command::new(shell_parts[0])
            .arg("-c")
            .arg(command)
            .status()?
    } else {
        Command::new(shell_parts[0])
            .args(&shell_parts[1..])
            .arg("-c")
            .arg(command)
            .status()?
    };

    if !status.success() {
        anyhow::bail!("Command failed with status: {}", status);
    }

    Ok(())
}

fn r#continue(ui: &mut UI) -> Result<()> {
    todo!("implement continue functionality")
}
