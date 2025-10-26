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
# opens config in editor so you can fill in your api key

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

## known bugs

- `please continue` (and history generally) is not implemented
- pressing Escape to get out of prompt is a bit wonky

## license

[non-violent public license](./LICENSE.md)
