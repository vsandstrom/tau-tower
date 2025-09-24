use hyper::Result;
use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};

use hyper::body::Bytes;

use crate::util::{validate_bos_and_tags, Headers};
use super::MTU;

/// Creates a Udp receiver listening to the sender of the ogg opus stream.
/// Appending the ogg opus blocks to a producer/consumer object.
pub async fn thread(
    tx: broadcast::Sender<Bytes>,
    udp_addr: impl tokio::net::ToSocketAddrs,
    header: Arc<Mutex<Headers>>
) -> Result<()> {
  use tokio::net::UdpSocket;
  let udp_socket = match UdpSocket::bind(udp_addr).await {
    Ok(s) => s,
    Err(e) => panic!("Could not connect to UDP port: {e}"),
  };
  let mut buf = [0u8; MTU];
  let mut temp_headers = vec!();
  let mut headers_parsed = false;
  let mut open_endpoint = true;
  loop {
    while let Ok(size) = udp_socket.recv(&mut buf).await {
      let page = Bytes::copy_from_slice(&buf[..size]);
      if validate_bos_and_tags(&page).is_ok() {
        temp_headers.push(page);
      } else {
        if !headers_parsed && let Ok(mut h) = header.lock() && let None = h.headers {
          h.prepare_headers(&temp_headers);
          headers_parsed = true;
        }
        match tx.send(page) {
          Ok(_) => { open_endpoint = true; },
          Err(e) => { 
            if open_endpoint {
              open_endpoint = false;
              eprintln!("could not open client stream: {e}"); 
            }
          }
        }
      }
    }
  }
}
