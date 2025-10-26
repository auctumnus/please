use std::io::{stdin, Read, Write};
use std::os::fd::{AsRawFd};

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Event, EventHandler, KeyCode, KeyEvent, Modifiers};
use colored::Colorize;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

use crate::config::Config;

pub enum UserAction {
    RunCommand(String),
    ProvideFeedback(String),
    EditCommand(String),
    Quit,
}

pub struct UI {
    editor: DefaultEditor,
    config: Config,
}

impl UI {
    pub fn new(config: Config) -> Result<Self> {
        let mut editor = DefaultEditor::new()?;
        editor.bind_sequence(Event::KeySeq(vec![KeyEvent(KeyCode::Esc, Modifiers::empty())]), EventHandler::Simple(rustyline::Cmd::Interrupt));
        Ok(Self { editor, config })
    }

    /// Display a command and get user action
    /// Returns:
    /// - UserAction::RunCommand if user presses Enter (run as-is)
    /// - UserAction::ProvideFeedback if user types feedback
    /// - UserAction::EditCommand if user edits the command and presses Enter
    /// - UserAction::Quit if user presses Ctrl-C or Ctrl-D
    pub fn display_command_and_get_action(&mut self, command: &str) -> Result<UserAction> {
        println!("{}", command);

        if !self.config.quiet {
            let message = format!("{} {} {}",
                "Press".bright_black().italic(),
                "Enter".bright_black(),
                "to run, type feedback to refine, or press arrow keys to edit the command.".bright_black().italic()
            );
            println!("{}", message);
        };
        std::io::stdout().flush()?;

        let stdin = stdin();
        let fd = stdin.as_raw_fd();
        
        // save original terminal settings
        let old_tio = Termios::from_fd(fd)?;
        let mut new_tio = old_tio;
        
        // raw mode: disable canonical and echo
        new_tio.c_lflag &= !(ICANON | ECHO);
        new_tio.c_cc[termios::VMIN] = 0;   // don't wait
        new_tio.c_cc[termios::VTIME] = 1;  // 0.1s timeout
        
        tcsetattr(fd, TCSANOW, &new_tio)?;
        
        let mut stdin = stdin.lock();
        let mut buf = [0u8; 1];
        
        loop {
            if stdin.read(&mut buf)? == 1 {
                if buf[0] == 27 {  // ESC
                    // try to read next byte with timeout
                    let mut next = [0u8; 1];
                    if stdin.read(&mut next)? == 0 {
                        // timeout! actual escape key
                        return Ok(UserAction::Quit);
                    } else if next[0] == b'[' {
                        // escape sequence, read third byte
                        let mut third = [0u8; 1];
                        let len = stdin.read(&mut third)?;
                        if len == 0 {
                            continue;
                        }
                        match third[0] {
                            // up, right, down, left
                            c @ (b'A' | b'B' | b'C' | b'D') => {
                                // transfer power over to readline
                                tcsetattr(fd, TCSANOW, &old_tio)?;
                                if self.config.quiet {
                                    print!("\r\033[2K"); // Clear current line
                                } else {
                                    // Cursor is at end of help message line
                                    print!("\r\x1b[2K\x1b[A\r\x1b[2K"); // clear help line
                                    print!("\r\x1b[2K\x1b[A\r\x1b[2K"); // clear command line
                                }
                                std::io::stdout().flush()?;

                                let initial = if c == b'D' {
                                    // left arrow, put cursor at end
                                    let (left, right) = command.split_at(command.len() - 1);
                                    (left, right)
                                } else {
                                    (command, "")
                                };

                                return self.get_from_readline(initial).map(UserAction::EditCommand);
                            },
                            _ => {
                                continue;
                            },
                        }
                    }
                } else {
                    let input_char = buf[0] as char;
                    if input_char == '\n' || input_char == '\r' {
                        // Enter pressed, run command
                        tcsetattr(fd, TCSANOW, &old_tio)?;
                        return Ok(UserAction::RunCommand(command.to_string()));
                    }

                    // we got a different character; move to input area and provide feedback
                    // transfer power over to readline
                    tcsetattr(fd, TCSANOW, &old_tio)?;
                    if !self.config.quiet {
                        print!("\r\x1b[2K\x1b[A\r\x1b[2K"); // clear help line
                        self.show_message("Refine:");

                    }
                    let input = format!("{}", input_char);
                    return self.get_from_readline((input.as_str(), "")).map(UserAction::ProvideFeedback);
                }
            }
        }
    }

    pub fn get_from_readline_with_prompt(&mut self, prompt: impl std::fmt::Display, initial: (&str, &str)) -> Result<String> {
        match self.editor.readline_with_initial(&prompt.to_string(), initial) {
            Ok(string) => Ok(string),
            Err(ReadlineError::Interrupted) => {
                std::process::exit(1);
            },
            Err(e) => {
                Err(e.into())
            }
        }
    }

    pub fn get_from_readline(&mut self, initial: (&str, &str)) -> Result<String> {
        self.get_from_readline_with_prompt("", initial)
    }

    pub fn show_error(&self, message: &str) {
        let error = format!("Error: {}", message).red().bold();
        eprintln!("{}", error);
    }

    pub fn show_message(&self, message: &str) {
        let message = message.italic().bright_black();
        println!("{}", message);
    }

    pub fn show_prompt(&mut self, prompt: impl std::fmt::Display) -> Result<String> {
        let input = self.get_from_readline_with_prompt(prompt, ("", ""))?;
        Ok(input)
    }
}
