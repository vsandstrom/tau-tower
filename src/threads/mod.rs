use hyper::Result;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::broadcast};
use hyper_util::{rt::TokioExecutor, server::conn::auto};
use std::sync::{Arc, Mutex};

use crate::http::handle_request;
use hyper::body::Bytes;

pub const MTU: usize = 1500;

pub struct Headers {
    pub headers: Option<Bytes>,
}

/// Creates a Udp receiver listening to the sender of the ogg opus stream.
/// Appending the ogg opus blocks to a producer/consumer object.
pub async fn udp_thread(
    tx: broadcast::Sender<Bytes>,
    udp_addr: impl tokio::net::ToSocketAddrs,
    header: &Arc<Mutex<Headers>>
) -> Result<()> {
  use tokio::net::UdpSocket;
  let udp_socket = match UdpSocket::bind(udp_addr).await {
    Ok(s) => s,
    Err(e) => panic!("Could not connect to UDP port: {e}"),
  };

  let mut buf = [0u8; MTU];
  let mut temp_headers = vec!();
  let mut headers_received = false;
  loop {
    'message: while let Ok(size) = udp_socket.recv(&mut buf).await {
      let page = Bytes::copy_from_slice(&buf[..size]);
      if !headers_received {
        if temp_headers.len() < 2 { 
          temp_headers.push(page);
          continue 'message;
        }
        else {
          if let Ok(mut h) = header.lock() && let None = h.headers {
            let bytes = prepare_headers(&temp_headers);
            h.headers = Some(bytes);
            headers_received = true;
          }
          continue 'message;
        }
      }

      if let Err(e) = tx.send(page) {
        eprintln!("oops, could not send into broadcast object: {e}")
      }
    }
  }
}

pub async fn http_thread(
    ip_addr: impl tokio::net::ToSocketAddrs,
    tx: broadcast::Sender<Bytes>,
    header: &Arc<Mutex<Headers>>
) {
  let listener = TcpListener::bind(ip_addr).await.unwrap();
  let tx_clone = tx.clone();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let _ = stream.set_nodelay(true);
    let io = TokioIo::new(stream);
    let tx_inner_clone = tx_clone.clone();
    let header_clone = header.clone();
    tokio::task::spawn(async move {
      if let Err(err) = auto::Builder::new(TokioExecutor::new())
        .serve_connection(io, service_fn(move |req| {
          handle_request(req, tx_inner_clone.clone(), header_clone.clone())
      }))
        .await
      {
        eprintln!("error serving connection: {}", err);
      }
    });
  }
}

fn prepare_headers(buf: &[Bytes]) -> Bytes {
  Bytes::copy_from_slice(&[&buf[0][..], &buf[1][..]].concat())
}
