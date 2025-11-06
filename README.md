# `please` cli

ask an llm for help in your closest unix shell!

## installation

with cargo:
```sh
git clone https://github.com/auctumnus/please
cd please
cargo install --path .
```

with nix:
```sh
nix shell github:auctumnus/please
```

## usage

```sh
$ please config
No config file found. Create one now? (y/n): y
# opens config in $EDITOR so you can fill in your api key

$ please find all .rs files modified in the last 2 days
Thinking...
find . -name "*.rs" -mtime -2
Press Enter to run, type feedback to refine, or press arrow keys to edit the command.

$ PLEASE_QUIET=1 please search for 'TODO' in all .py files and count occurrences
grep -r "TODO" --include="*.py" | wc -l

$ PLEASE_MODEL="anthropic/claude-3.7-sonnet" please celebrate
Thinking...
echo -e "\n\033[1;32m*\033[0m \033[1;31m*\033[0m \033[1;34m*\033[0m \033[1;33mCelebration!\033[0m \033[1;34m*\033[0m \033[1;31m*\033[0m \033[1;32m*\033[0m\n"
Press Enter to run, type feedback to refine, or press arrow keys to edit the command.

* * * Celebration! * * *
```

## configuration

place a [json5](https://json5.org/) file in `$XDG_CONFIG_HOME/please/config.json{,5}` with this schema:

```json5
// please cli configuration
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
    // "response-format": "json_schema",

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
```

configuration can also be passed in through environment variables; the equivalent for each is
`PLEASE_$VAR` where `VAR` is a an UPPER_SNAKE_CASE version of the variable name (so "api-key" is PLEASE_API_KEY, "prompts.command" is PLEASE_PROMPTS_COMMAND)


## known bugs

- `please continue` (and history generally) is not implemented
- pressing Escape to get out of prompt is a bit wonky

## license

[non-violent public license](./LICENSE.md)
