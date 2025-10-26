`please`: cli to ask an openrouter model for a command in sh, and then run it

examples:
```
# ask for help with ffmpeg

$ please convert all pngs to jpegs

| for file in *.jpeg *.jpg; do ffmpeg -i "$file" "${file%.*}.png"; done

# press enter to run, ctrl-c or ctrl-d to quit without running
# if you type, you can communicate back "no, not that, can you instead ..."
# eg if you then type "convert to webm instead", it'll display

| for file in *.jpeg *.jpg; do ffmpeg -i "$file" "${file%.*}.webm"; done

# if you use arrow keys you can go edit it yourself as well; you can press escape to go back to "prompt mode" after that (characters will be typed into the prompt instead of the command)
# but enter in "edit mode" will run the command directly
```

we might need an intermediate call to ask to improve the prompt, but ideally an llm can just figure that out anyways

in `$XDG_CONFIG_DIR/please/config.json{,5}` (json5 is loaded first, supercedes json, regular .json also gets parsed as json5 since its backwards compatible, default configs are written as json5):

```
// please config file; this is read as json5
{
    // used to authenticate with the api endpoint (required)
    "api-key": "<API_KEY>",
    
    // model requested from the api (defaults to "anthropic/claude-haiku-4.5")
    "model": "anthropic/claude-haiku-4.5",
    
    // path to the shell to execute commands in (defaults to "/usr/bin/env sh")
    "shell": "/usr/bin/env sh",

    // openai api compatible endpoint (defaults to "https://openrouter.ai/api/v1")
    "endpoint": "https://openrouter.ai/api/v1",
    
    "prompts": {
        // prompt for requesting the initial command
        // you can include some variables, using an environment-variable-like syntax:
        // - `$SHELL`: replaced with the shell defined in the config
        // default is below
        "command": "You are an expert in the the Linux shell. The user would like to perform a task in the shell. \
Please return a command compatible with the user's shell (it will be ran with `$SHELL`, preferring single-line \ 
solutions to multi-line solutions.)"
    },
}
/* -*- mode: json5 -*- */
/* vim: set ft=json5: */
```

all config can also be passed in as environment variables, with each being set by a variable like `PLEASE_$NAME`, where `$NAME` is an uppercased snake case version of the environment variable's name. things like `"prompts.command"` become "PLEASE_PROMPTS_COMMAND"

for actually requesting, may be a good idea to use structured outputs?

/* -*- mode: md -*- */
/* vim: set ft=md: */

