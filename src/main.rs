#![deny(unused_crate_dependencies)]
mod server;
mod threads;

use std::net::{Ipv4Addr, SocketAddr};
use tokio::sync::broadcast;
use tokio::task;
use std::sync::{Arc, Mutex};
use hyper::body::Bytes;

use crate::threads::{
  http, udp, ws,
  Headers
};

enum ServerMode {
  WebSocket,
  Udp
}

// TODO: Change for sane defaults and config-loaded values
const UDP: u16 = 8001;
const PORT: u16 = 8002;
const END_POINT: &str = "tau.ogg";

const MODE: ServerMode = ServerMode::WebSocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let local_ip = std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
  let remote_ip = std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

  let headers = Arc::new(Mutex::new(Headers{headers: None})); 
  let headers_clone = headers.clone();

  // used to send ogg opus blocks between Udp thread to WebSocket thread
  let (tx, _) = broadcast::channel::<Bytes>(128);
  let tx_clone = tx.clone();
  // let tx_clone2 = tx.clone();

  let local_addr = SocketAddr::new(local_ip, PORT);
  let remote_addr = SocketAddr::new(remote_ip, UDP);

  // TODO: swap to value from config
  let end_point = format!("/{END_POINT}");
  let mount: Arc<str> = Arc::from(end_point);
  let mount_clone = mount.clone();

  match MODE {
    ServerMode::Udp => {
      // receive audio
      task::spawn(async move {
        udp::thread(tx_clone, remote_addr, headers_clone).await.unwrap();
      });

    },
    ServerMode::WebSocket => {
      // receive audio
      task::spawn(async move {
        ws::thread(tx_clone, remote_addr, local_addr, headers_clone).await;
      });
    }
  }

  // serve audio stream
  task::spawn(async move {
    http::thread(local_addr, tx, &headers, mount_clone).await
  });

  println!("Running on http://{}:{}{mount}", local_addr.ip(), local_addr.port());

  futures_util::future::pending::<()>().await;
  Ok(())
}
