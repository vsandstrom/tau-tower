[![Compiles on macOS (Intel / Silicon), Ubuntu and NixOS](https://github.com/tau-org/tau-tower/actions/workflows/rust.yml/badge.svg?event=pull_request)](https://github.com/tau-org/tau-tower/actions/workflows/rust.yml)

## Usage:
This tool is built for livestreaming audio to the world wide web, broadcasting 
a audio stream from an instance of 
[`tau-radio`](https://github.com/tau-org/tau-radio), the accompanying software.

Modelled after the Icecast software, it serves a html audio stream that can be
used in a audio tag on any other website.

You should run this on a remote server, such as a AWS, Digital Ocean or any VPS
with the correct priviledges. 
- Note that the smallest available Digital Ocean 'Droplet' does not have enough
  RAM to build this project locally. The workaround is to build for that 
  architecture using ex: `cargo build --target x86_64-unknown-linux-gnu`. 

---

To install:
```bash
$ cargo install --git https://github.com/tau-org/tau-tower
```

The first time using the tool, it will search your system for a config file. 
It looks for it in the directory:
```bash
$ $HOME/.config/tau/tower.toml
```

If there is no config file located there, you will be prompted to create one. 

```toml
# username and password are NOT secure, they only
# link a tauradio and tautower service together
username = "username" 
password = "emanresu" 

# Sets the listening port, to which the source stream is transmitted
listen_port = 8000      

# Sets the broadcast port, from which the stream will be accessable
mount_port = 8001       

# Sets the server http endpoint - http://localhost:8001/tau.ogg
mount = "tau,ogg"       

# Optional: 
# Sets which other sites are able to rebroadcast the stream
# "*" allowes all, adding "http://localhost:4000" to list is redundant
cors_allow_list = ["*", "http://localhost:4000"]
```

<!-- [![asciicast](https://asciinema.org/a/JqdeXeILf0lALG34pZzAarmih.svg)](https://asciinema.org/a/JqdeXeILf0lALG34pZzAarmih) -->

If you want to temporarily overwrite the config, you are able to pass arguments.

```bash
# Ex: Uses temporary credentials, and disables the local recording. 
$ tau-tower \
  --listen-port <listen-port> \
  --mount-port <mount-port> \
  --cors-allow-list "*"
```

### Dependencies

**On Linux** (using apt):
```bash
$ sudo apt update
$ sudo apt install build-essential
```
