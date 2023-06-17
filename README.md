<h1 align="center">
Multihook
</h1>
<p align="center">
    <a href="https://crates.io/crates/multihook">
        <img src="https://img.shields.io/crates/v/multihook?style=for-the-badge">
    </a>
</p>

Multihook is an easy to configure webhook server.

## Installation

With cargo:
```
cargo install multihook
```

## Usage

Just run it via systemd or smth.

```
multihook
```

## Config

The config allows you to configure actions for each endpoint. The config is most likely
stored in `~/.config/multihook` and on Windows maybe in the APPDATA directory (?).
After running the program for the first time the config directory and config file should be created.

```toml
[server]
address = '127.0.0.1:8080'

[hooks]
# executed before all endpoint actions
pre_action = "echo 'pre action'"
# executed after all endpoint actions
post_action = "echo 'post action'"
# executed when an action fails
err_action = "echo \"Hook $HOOK_NAME failed with error: $HOOK_ERROR\""

# the name needs to be unique
[endpoints.ls]
# the path needs to be unique
path = "path/on/the/server"
# a command or a path to the script
action = "ls {{$.filepath}}"
# allows multiple instances of this action to run concurrently
allow_parallel = true
# additional hooks on endpoint-level
hooks = {pre_action = "echo 'before something bad happens'"}

[endpoints.error]
path = "error"
action = "echo '{{$.books.*.title}}'"
# Validate secrets according to different parsing rules
# Currently only GitHub secrets are supported
secret = { value = "my secret", format = "GitHub"}

[endpoints.testscript]
path = "script"
action = "/home/trivernis/.local/share/multihook/test-script.sh"
allow_parallel = false
# doesn't wait for the command to finish and returns a http response directly
# This setting can be useful if your action takes a very long time to run and would
# cause a timeout
run_detached = true
```

The configured `action` is either a script file or a command.
In both cases placeholders with the syntax `{{query}}` can be used. The query
is the path to required values in the json body of the request. The request body
will also be provided in the environment variable `HOOK_BODY`.

## License

GPL-3
