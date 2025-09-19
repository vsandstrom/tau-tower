

use clap::Parser;

// use crate::StreamType;
use crate::config::TauConfigError;
use is_ip::is_ip;

#[derive(Parser)]
#[command(name = "Tau")]
#[command(version = "0.0.1")]
#[command( about = "Hijacks chosen audio device, encodes audio into Ogg Opus and streams to IceCast server")]
pub(crate) struct Args {
    /// IceCast server username
    #[arg(long)]
    pub username: Option<String>,

    /// IceCast server password
    #[arg(long)]
    pub password: Option<String>,

    /// Stream port
    #[arg(short='l', long, value_parser=|p: &str| {
      validate_port(parse_port(p).unwrap())
    })]
    pub listen_port: Option<u16>,
    
    /// Stream port
    #[arg(short='p', long, value_parser=|p: &str| {
      validate_port(parse_port(p).unwrap())
    })]
    pub mount_port: Option<u16>,

    #[arg(short, long)]
    pub mount: Option<String>,

    #[arg(long)]
    pub reset_config: bool,
    // #[arg(long, value_parser=validate_stream_type)]
    // pub stream_mode: crate::StreamType
}

pub fn validate_ip(url: String) -> Result<String, TauConfigError> {
  if !is_ip(&url) {
    return Err(TauConfigError::InvalidIp(url));
  }
  Ok(url)
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
