use hyper::Result;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::broadcast};
use hyper_util::{rt::TokioExecutor, server::conn::auto};
use std::sync::{Arc, Mutex};

use crate::http::handle_request;
use hyper::body::Bytes;
use tokio_tungstenite::connect_async;
use futures_util::StreamExt;

pub const MTU: usize = 1500;

pub struct Headers {
    pub headers: Option<Bytes>,
}

/// Creates a WebSocket receiver listening to the sender of the ogg opus stream.
/// Appending the ogg opus blocks to a producer/consumer object.
pub async fn ws_thread(
    tx: broadcast::Sender<Bytes>,
    src_addr: (&str, u16),
    header: &Arc<Mutex<Headers>>
) {
  let url = format!("ws://{}:{}", src_addr.0, src_addr.1);
  let mut temp_headers: Vec<Bytes> = vec!();
  let mut headers_parsed = false;
  loop {
    if let Ok((mut ws_stream, _)) = connect_async(&url).await {
      while let Some(msg) = ws_stream.next().await  {
        let page = msg.unwrap().into_data();
        if validate_bos_and_tags(&page).is_ok() {
          temp_headers.push(page);
        } else {
          if !headers_parsed && let Ok(mut h) = header.lock() && let None = h.headers {
            h.headers = Some(prepare_headers(&temp_headers));
            headers_parsed = true;
          }
          if let Err(e) = tx.send(page) {
            eprintln!("oops, could not send into broadcast object: {e}")
          }
        }
      }
    }
  }
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
  let mut headers_parsed = false;
  loop {
    while let Ok(size) = udp_socket.recv(&mut buf).await {
      let page = Bytes::copy_from_slice(&buf[..size]);
      if validate_bos_and_tags(&page).is_ok() {
        temp_headers.push(page);
      } else {
        if !headers_parsed && let Ok(mut h) = header.lock() && let None = h.headers {
          h.headers = Some(prepare_headers(&temp_headers));
          headers_parsed = true;
        }
        if let Err(e) = tx.send(page) {
          eprintln!("oops, could not send into broadcast object: {e}")
        }
      }
    }
  }
}

pub async fn http_thread(
    ip_addr: impl tokio::net::ToSocketAddrs,
    tx: broadcast::Sender<Bytes>,
    header: &Arc<Mutex<Headers>>,
    mount: Arc<str>
) {
  let listener = TcpListener::bind(ip_addr).await.unwrap();
  let tx_clone = tx.clone();
  let mount = mount.clone();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let _ = stream.set_nodelay(true);
    let io = TokioIo::new(stream);
    let tx_inner_clone = tx_clone.clone();
    let header_clone = header.clone();
    let mount_clone = mount.clone();
    tokio::task::spawn(async move {
      if let Err(err) = auto::Builder::new(TokioExecutor::new())
        .serve_connection(io, service_fn(move |req| {
          handle_request(req, tx_inner_clone.clone(), header_clone.clone(), mount_clone.clone())
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

fn validate_bos_and_tags(data: & Bytes) -> core::result::Result<&Bytes, ()> {
  let n_segs = data[26] as usize;
  let offset = 27+n_segs;
  if data.len() < 27 + 8 { return Err(()) }
  if matches!(&data[offset..offset+8], b"OpusTags" | b"OpusHead") {
    return Ok(data);
  }
  Err(())
}

fn validate_bos(data: &Bytes) -> bool {
  if data[5] == 0x2 { return true; }
  false
}

fn validate_eos(data: &Bytes) -> bool {
  if data[5] == 0x4 {return true; }
  false
}
