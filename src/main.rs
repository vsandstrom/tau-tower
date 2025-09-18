mod server;
mod threads;
mod config;
mod args;

use std::net::{Ipv4Addr, SocketAddr};
use tokio::sync::broadcast;
use tokio::task;
use std::sync::{Arc, Mutex};
use hyper::body::Bytes;
use clap::Parser;

use crate::threads::{
  http,
  udp,
  ws,
  Headers
};
use crate::config::Config;
use crate::args::Args;
use std::str::FromStr;

enum ServerMode {
  WebSocket,
  Udp
}

struct Credentials {
  pub username: String,
  pub password: String,
  pub broadcast_port: u16
}

// TODO: Change for sane defaults and config-loaded values
// const UDP: u16 = 8888;
const PORT: u16 = 8002;
const END_POINT: &str = "tau.ogg";

const MODE: ServerMode = ServerMode::WebSocket;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let config = Config::load_or_create(args.reset_config).map(|c| c.merge_cli_args(&args))?;
  let creds = Credentials{
    username: config.username.clone(), 
    password: config.password.clone(),
    broadcast_port: config.mount_port
  };

  let headers = Arc::new(Mutex::new(Headers{headers: None})); 
  // used to send ogg opus blocks between Udp thread to WebSocket thread
  let (tx, _) = broadcast::channel::<Bytes>(128);

  let ip = Ipv4Addr::from_str(&config.url).unwrap();
  let server_addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.mount_port);
  let listen_addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.listen_port);

  // TODO: swap to value from config
  let end_point = format!("/{END_POINT}");
  let mount: Arc<str> = Arc::from(end_point);
  let mount_clone = mount.clone();

  match MODE {
    ServerMode::Udp => {
      let tx_clone = tx.clone();
      let headers_clone = headers.clone();
      // receive audio
      task::spawn(async move {
        udp::thread(tx_clone, listen_addr, headers_clone).await.unwrap();
      });

    },
    ServerMode::WebSocket => {
      let tx_clone = tx.clone();
      let headers_clone = headers.clone();
      // receive audio
      task::spawn(async move {
        ws::thread(tx_clone, listen_addr, creds, headers_clone).await;
      });
    }
  }

  // serve audio stream
  task::spawn(async move {
    http::thread(server_addr, tx, &headers, mount_clone).await
  });

  println!("Serving stream on http://{}:{}{}", ip, config.mount_port, mount);

  futures_util::future::pending::<()>().await;
  Ok(())
}
