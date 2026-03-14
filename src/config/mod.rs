use dialoguer::{Input, Password};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use inline_colorization::{color_reset, color_bright_red, color_bright_yellow};
use crate::util::ip::{validate_ip, validate_port, validate_endpoint};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub ip: String,
    pub listen_port: u16,
    pub mount_port: u16,
    pub cors_port: Option<u16>,
    pub mount: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TauConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parsing error: {0}")]
    TomlRead(#[from] toml::de::Error),
    
    #[error("toml writing error: {0}")]
    TomlWrite(#[from] toml::ser::Error),

    #[error("invalid IP: {0}")]
    InvalidIp(String),

    #[error("invalid port number: {0}")]
    InvalidPort(String),

    #[error("invalid endpoint formatting: {0}")]
    InvalidEndpoint(String),

    #[error("user input error: {0}")]
    Input(String),
}

impl Config {
  fn get_config_path() -> PathBuf {
    let local_dir = PathBuf::new().join("tau").join("tower.toml");
    match (std::env::var("XDG_CONFIG_HOME"), std::env::var("HOME")) {
      // XDG_CONFIG_HOME
      (Ok(path), _) => PathBuf::from(path).join(local_dir),
      // HOME
      (_, Ok(path)) => PathBuf::from(path).join(".config").join(local_dir),
      // Fallback
      _ => PathBuf::from("tower.toml"),
    }
  }

  /// Merges local config.toml with current CLI arguments if there are any.
  pub fn merge_cli_args(mut self, args: &crate::args::Args) -> Self {
    if let Some(username) = &args.username {
      self.username.clone_from(username);
    }
    if let Some(password) = &args.password {
      self.password.clone_from(password);
    }
    if let Some(listen_port) = args.listen_port {
      self.listen_port = listen_port;
    }
    if let Some(mount_port) = args.mount_port {
      self.mount_port = mount_port;
    }
    if let Some(endpoint) = &args.mount {
      self.mount.clone_from(endpoint);
    }
    if args.asciinema_port.is_some() {
      self.cors_port = args.asciinema_port;
    }
    self
  }

  fn load_config(path: &PathBuf) -> Result<Self, TauConfigError> {
    let settings = fs::read_to_string(path)?; //.expect("could not read config file");
    match toml::from_str(&settings) {
      Ok(config) => Ok(config),
      Err(e) => Err(TauConfigError::TomlRead(e)),
    }
  }

  /// Creates an instance of Config, and reads from the saved `config.toml` file stored on disc.
  /// If no `config.toml` file can be found, it prompts the user to enter one.
  pub fn load_or_create(reset: bool) -> Result<Self, TauConfigError> {
    let path = Self::get_config_path();
    if path.exists() && !reset {
      Self::load_config(&path)
    } else {
      println!(
        "\n{color_bright_red}No config found at '{}'. Let's create one: {color_reset}",
        path.display()
      );
      println!("{color_bright_yellow}Credentials must correspond to the source stream config{color_reset}\n");
      let username: String = Input::new()
        .with_prompt(format!("{color_bright_yellow}Username{color_reset}"))
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;

      let password: String = Password::new()
        .with_prompt(format!("{color_bright_yellow}Password{color_reset}"))
        .interact()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;

      let ip: String = Input::new()
        .with_prompt(format!("{color_bright_yellow}Public IP for server{color_reset}"))
        .default("127.0.0.1".to_string())
        .interact_text()
        .map_err(|e| TauConfigError::InvalidIp(e.to_string()))
        .and_then(validate_ip)?;

      let listen_port: u16 = Input::new()
        .with_prompt(format!("{color_bright_yellow}Source port{color_reset}"))
        .default(8000)
        .interact_text()
        .map_err(|e| TauConfigError::InvalidPort(e.to_string()))
        .and_then(validate_port)?;
      
      let mount_port: u16 = Input::new()
        .with_prompt(format!("{color_bright_yellow}Broadcast port{color_reset}"))
        .default(8001)
        .interact_text()
        .map_err(|e| TauConfigError::InvalidPort(e.to_string()))
        .and_then(validate_port)?;

      let mount = Input::new()
        .with_prompt(format!("{color_bright_yellow}Mount endpoint{color_reset}"))
        .default("tau.ogg".to_string())
        .interact_text()
        .map_err(|e| TauConfigError::InvalidEndpoint(e.to_string()))
        .and_then(|x| validate_endpoint(x.as_ref()))?;

      let asciinema_port: String = Input::new()
        .with_prompt(format!("{color_bright_yellow}Optional Asciinema Server port{color_reset}"))
        .allow_empty(true)
        .interact_text()
        .map_err(|e| TauConfigError::InvalidPort(e.to_string()))?;

      let asciinema_port = if asciinema_port.is_empty() {
        None
      } else if let Ok(port) = asciinema_port.parse::<u16>() 
        && validate_port(port).is_ok() {
        Some(port)
      } else {
        None
      };

      let config = Self {
        username,
        password,
        ip,
        listen_port,
        mount_port,
        cors_port: asciinema_port,
        mount,
      };

      if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
      }

      match toml::to_string_pretty(&config) {
        Ok(toml_string) => {
          fs::write(&path, toml_string)?;
          crate::util::ui::config_file_created_info(&path);
          Ok(config)
        },
        Err(e) => {
          Err(TauConfigError::TomlWrite(e))
        }
      }
    }
  }
}
