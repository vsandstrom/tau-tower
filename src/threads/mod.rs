use futures_util::{SinkExt, StreamExt};
use hyper::Result;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::broadcast, task};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use hyper_util::{rt::TokioExecutor, server::conn::auto};
use std::sync::{Arc, Mutex};

use crate::http::handle_request;

pub const MTU: usize = 1500;

pub struct Headers {
    pub headers: Option<Vec<Vec<u8>>>,
}

/// Creates a Udp receiver listening to the sender of the ogg opus stream.
/// Appending the ogg opus blocks to a producer/consumer object.
pub async fn udp_thread(
    tx: broadcast::Sender<Vec<u8>>,
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
  let mut headers_received = false;
  let mut temp_page = vec!();
  loop {
    'message: while let Ok(size) = udp_socket.recv(&mut buf).await {
      if !headers_received {
        if temp_headers.len() < 2 { 
          temp_headers.push(buf[..size].to_vec());
          continue 'message;
        }
        else {
          if let Ok(mut h) = header.lock() && let None = h.headers {
            h.headers = Some(temp_headers[..].to_vec());
            headers_received = true;
          }
          continue 'message;
        }
      }

      // TODO: LOOK INTO OGG HEADER TO SEE WHAT IS WHAT
      temp_page.extend_from_slice(&buf[..size]);
      while let Some((page, rest)) = validate_opus_block(&temp_page) {
        if let Err(e) = tx.send(page) {
          eprintln!("oops, could not send into broadcast object: {e}")
        }
        temp_page = rest;
      };
    }
  }
}

/// Handles connections from clients and upgrades to Websocket connections.
/// Reads from the producer/consumer object with the ogg opus blocks, and spawns a
/// new async task for each client, and sends the ogg opus blocks to frontend socket.
pub async fn ws_thread(
  tx: broadcast::Sender<Vec<u8>>,
  socket_addr: impl tokio::net::ToSocketAddrs,
  header: Arc<Mutex<Headers>>
) -> Result<()> {
  let ws_listener = TcpListener::bind(socket_addr).await.unwrap();
  while let Ok((stream, _)) = ws_listener.accept().await {
    let ws = accept_async(stream).await.unwrap();
    let (mut tx_ws, mut rx_ws) = ws.split();
    let mut rx = tx.subscribe();

    let headers = header.lock().map(|h| h.headers.clone()).unwrap();


    if let Some(h) = &headers {
      let inner = h.to_vec();
      for buf in inner {
        println!("{:?}", &buf);
        let msg = Message::binary(buf);
        if tx_ws.send(msg).await.is_err() {
          eprintln!("could not send ogg headers");
        } 
      }
    }

    let mut send_task = task::spawn(async move {
      while let Ok(buf) = rx.recv().await {
        // println!("{buf:?}");
        let msg = Message::binary(buf);

        if tx_ws.send(msg).await.is_err() {
          break;
        }
      }
    });

    // keeping the websocket alive with fw polling
    let mut recv_task = task::spawn(async move { 
      while let Some(Ok(_msg)) = rx_ws.next().await {
        eprintln!("{_msg}");
      }
    });

    task::spawn(async move {
      tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),

      }
    });
  }

  Ok(())
}

pub async fn http_thread(
    ip_addr: impl tokio::net::ToSocketAddrs,
    tx: broadcast::Sender<Vec<u8>>,
    header: Arc<Mutex<Headers>>
) {
  let listener = TcpListener::bind(ip_addr).await.unwrap();
  let tx_clone = tx.clone();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
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

fn validate_opus_block(data: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
  let len = data.len();

  if len < 27 { return None }

  let segments = data[26] as usize;
  let needed = 27 + segments;

  if len < needed { return None }

  let body_len: usize = data[27..(27+segments)]
    .iter()
    .map(|&b| b as usize)
    .sum();

  if data.len() < (body_len + needed) {
    return None;
  }

  let total = body_len + needed;
  let (page, rest) = data.split_at(total);
  Some((page.to_vec(), rest.to_vec()))
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_opus_block_spit() {
    let test_data = b"lkjasdvbyebceOggSknvad".to_vec();
    assert!(validate_opus_block(&test_data).map(|(a, b)| {
      println!("{a:?}  {b:?}");
      if a.is_empty() && b.is_empty() {
        return false
      } 
      true
    }).unwrap())

  }
}


