#![deny(unused_crate_dependencies)]
mod http;
mod threads;

use std::net::{Ipv4Addr, SocketAddr};
use tokio::sync::broadcast;
use tokio::task;
use std::sync::{Arc, Mutex};
use hyper::body::Bytes;

use crate::threads::{
  http_thread, udp_thread,
  Headers
};

// TODO: Change for sane defaults and config-loaded values
const UDP: u16 = 8001;
const PORT: u16 = 8002;
const END_POINT: &str = "/tau.ogg";

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

  let ip_addr = SocketAddr::new(local_ip, PORT);
  let udp_addr = SocketAddr::new(remote_ip, UDP);

  // receive audio
  task::spawn(async move {
    udp_thread(tx_clone, udp_addr, &headers_clone).await.unwrap();
  });

  // serve audio stream
  task::spawn(async move {
    http_thread(ip_addr, tx, &headers).await
  });

  println!("Running on http://{ip_addr}{END_POINT}");

  futures_util::future::pending::<()>().await;
  Ok(())
}
