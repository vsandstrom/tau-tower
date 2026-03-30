use clap::Parser;
use crate::util::ip::{parse_origin, parse_port, validate_endpoint, validate_port};

#[derive(Parser)]
#[command(name = "tau-tower")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command( about = "Webradio server, distributes audio stream from a tau-radio client")]
pub struct Args {
    /// Webradio username
    #[arg(short, long)]
    pub username: Option<String>,

    /// Webradio password
    #[arg(short, long)]
    pub password: Option<String>,

    /// Stream port
    #[arg(short='l', long, value_parser=|p: &str| { validate_port(parse_port(p).unwrap()) })]
    pub listen_port: Option<u16>,
    
    /// Stream port
    #[arg(short='b', long, value_parser=|p: &str| { validate_port(parse_port(p).unwrap()) })]
    pub broadcast_port: Option<u16>,

    #[arg(short='a', long, value_parser=|s: &str| { parse_origin(s) })]
    pub cors_allow_list: Option<Vec<String>>,

    #[arg(short='e', long, value_parser=|e: &str| { validate_endpoint(e) })]
    pub broadcast_endpoint: Option<String>,

    #[arg(long)]
    pub reset_config: bool,
}

