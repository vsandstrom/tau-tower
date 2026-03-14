use clap::Parser;
use crate::util::ip::{parse_origin, parse_port, validate_endpoint, validate_port};

#[derive(Parser)]
#[command(name = "tau-tower")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command( about = "Webradio server, distributes audio stream from a tau-radio client")]
pub struct Args {
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

    #[arg(short='a', long, value_parser=|s: &str| { parse_origin(s) })]
    pub cors_allow_list: Option<Vec<String>>,

    #[arg(short, long, value_parser=|m: &str| { validate_endpoint(m) })]
    pub mount: Option<String>,

    #[arg(long)]
    pub reset_config: bool,
}

