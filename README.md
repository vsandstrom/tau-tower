[![Compiles on macOS (Intel / Silicon), Ubuntu and NixOS](https://github.com/tau-org/tau-tower/actions/workflows/rust.yml/badge.svg?event=pull_request)](https://github.com/tau-org/tau-tower/actions/workflows/rust.yml)

This project is funded through [NGI Zero Core](https://nlnet.nl/core), a fund established by [NLnet](https://nlnet.nl) with financial support from the European Commission's [Next Generation Internet](https://ngi.eu) program. Learn more at the [NLnet project page](https://nlnet.nl/project/Tau).

[<img src="https://nlnet.nl/logo/banner.png" alt="NLnet foundation logo" width="20%" />](https://nlnet.nl)
[<img src="https://nlnet.nl/image/logos/NGI0_tag.svg" alt="NGI Zero Logo" width="20%" />](https://nlnet.nl/core)

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
broadcast-port = 8001       

# Sets the server http endpoint - http://localhost:8001/tau.ogg
broadcast-endpoint = "tau.ogg"       

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
  --broadcast-port <broadcast-port> \
  --cors-allow-list "*"
```

### Dependencies

**On Linux** (using apt):
```bash
$ sudo apt update
$ sudo apt install build-essential
```

### Streaming Pipeline

[`tau-radio`](https://github.com/tau-org/tau-radio) runs on your local
machine, and captures sound from the audio device on your system. The defaults
are `BlackHole 2ch` on macOS, and `pipewire` on Linux, though these can be
overwritten by the config or in the CLI arguments.

The captured audio is then streamed to the
[`tau-tower`](https://github.com/tau-org/tau-tower), which should run on a
remote server. This server application exposes a audio media stream that can be
consumed by many clients, as a web radio. 

Alongside `tau-tower` should run an instance of [`Asciinema`](https://github.com/asciinema/asciinema) 
which can use the live audio stream as background to a live terminal stream, by
setting the broadcast endpoint URL as a media source in the streams settings.

```
https://example.com:8002/tau.ogg
```
 
For this to work, the Asciinema origin must be added to `cors_allow_list` in `tower.toml`:
 
```toml
cors_allow_list = ["https://example.com:4000"]
```
```

```
[your computer]                        [remote server]
  tau-radio  ──── internet ──▶  Caddy  ──▶  tau-tower  ──▶  Asciinema
(audio capture)                (proxy)     (broadcaster)    (stream host)
```

> For TLS termination and reverse proxy setup, see [Proxy Setup (Caddy)](docs/proxy-setup.md).
