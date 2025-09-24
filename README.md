## Usage:
This tool is built for livestreaming audio to the world wide web, broadcasting 
a audio stream from an instance of 
[`tau-radio`](https://github.com/tau-org/tau-radio) the accompanying software.

Modelled after the Icecast software, it serves a html audio stream that can be
used in a audio tag on any other website.

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

If you want to temporarily overwrite the config, you are able to pass arguments.

```bash
# Ex: Uses temporary credentials, and disables the local recording. 
$ tau \
  --listen-port <listen-port> \
  --mount-port <mount-port> \
```


