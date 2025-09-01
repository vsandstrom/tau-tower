#![deny(unused_crate_dependencies)]
mod http;
mod threads;

use std::net::{Ipv4Addr, SocketAddr};
use tokio::sync::broadcast;
use tokio::task;

use crate::threads::{http_thread, udp_thread, ws_thread};

// TODO: Change for sane defaults and config-loaded values
const UDP: u16 = 8001;
const PORT: u16 = 8002;
const SOCKET: u16 = 9001;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let local_ip = std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
  let remote_ip = std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

  // used to send ogg opus blocks between Udp thread to WebSocket thread
  let (tx, _) = broadcast::channel::<Vec<u8>>(1024);
  let tx_clone = tx.clone();

  // handle udp listener
  task::spawn(async move {
    let udp_addr = SocketAddr::new(remote_ip, UDP);
    udp_thread(tx_clone, udp_addr).await.unwrap();
  });

  // handle websocket thread
  task::spawn(async move {
    let socket_addr = SocketAddr::new(local_ip, SOCKET);
    ws_thread(tx, socket_addr).await.unwrap();
  });

  let ip_addr = SocketAddr::new(local_ip, PORT);
  // handle http serve
  task::spawn(async move {
    http_thread(ip_addr).await
  });

  println!("Running on http://{ip_addr}");

  futures_util::future::pending::<()>().await;
  Ok(())
}
