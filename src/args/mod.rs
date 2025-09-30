

use clap::Parser;

// use crate::StreamType;
use crate::config::TauConfigError;
use is_ip::is_ip;

#[derive(Parser)]
#[command(name = "tau-tower")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command( about = "Webradio server, distributes audio stream from a tau-radio client")]
pub(crate) struct Args {
    /// Webradio username
    #[arg(long)]
    pub username: Option<String>,

    /// Webradio password
    #[arg(long)]
    pub password: Option<String>,

    /// Stream port
    #[arg(short='l', long, value_parser=|p: &str| { validate_port(parse_port(p).unwrap()) })]
    pub listen_port: Option<u16>,
    
    /// Stream port
    #[arg(short='p', long, value_parser=|p: &str| { validate_port(parse_port(p).unwrap()) })]
    pub mount_port: Option<u16>,

    #[arg(short, long)]
    pub mount: Option<String>,

    #[arg(long)]
    pub reset_config: bool,
}

pub fn validate_ip(ip: String) -> Result<String, TauConfigError> {
  if !is_ip(&ip) {
    return Err(TauConfigError::InvalidIp(ip));
  }
  Ok(ip)
}

fn parse_port(p: &str) -> Result<u16, TauConfigError> {
  p.parse::<u16>()
    .map_err(|e| TauConfigError::Input(format!("Unable to parse as number: {e}")))
}

pub fn validate_port(port: u16) -> Result<u16, TauConfigError> {
  if !(1..=0xFFFF).contains(&port) {
    return Err(TauConfigError::InvalidPort(port));
  }
  Ok(port)
}
