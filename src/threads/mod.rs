use futures_util::{SinkExt, StreamExt};
use hyper::Result;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::broadcast, task};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use hyper_util::{rt::TokioExecutor, server::conn::auto};

use crate::http::serve;

const MTU: usize = 1500;

/// Creates a Udp receiver listening to the sender of the ogg opus stream.
/// Appending the ogg opus blocks to a producer/consumer object.
pub async fn udp_thread(
  tx: broadcast::Sender<Vec<u8>>,
  udp_addr: impl tokio::net::ToSocketAddrs,
) -> Result<()> {
  use tokio::net::UdpSocket;
  let socket = match UdpSocket::bind(udp_addr).await {
    Ok(s) => s,
    Err(e) => panic!("Could not connect to UDP port: {e}"),
  };

  let mut buf = [0u8; MTU];
  loop {
    while let Ok(size) = socket.recv(&mut buf).await {
      let msg = buf[..size].to_vec();
      tx.send(msg).unwrap();
    }
  }
}

/// Handles connections from clients and upgrades to Websocket connections.
/// Reads from the producer/consumer object with the ogg opus blocks, and spawns a
/// new async task for each client, and sends the ogg opus blocks to frontend socket.
pub async fn ws_thread(
  tx: broadcast::Sender<Vec<u8>>,
  socket_addr: impl tokio::net::ToSocketAddrs,
) -> Result<()> {
  let ws_listener = TcpListener::bind(socket_addr).await.unwrap();
  while let Ok((stream, _)) = ws_listener.accept().await {
    let ws = accept_async(stream).await.unwrap();
    let (mut tx_ws, mut rx_ws) = ws.split();
    let mut rx = tx.subscribe();

    let mut send_task = task::spawn(async move {
      while let Ok(buf) = rx.recv().await {
        let msg = Message::binary(buf);
        let _ = tx_ws.send(msg).await;
      }
    });

    // keeping the websocket alive with fw polling
    let mut recv_task = task::spawn(async move { 
      while let Some(Ok(_msg)) = rx_ws.next().await {}
    });

    tokio::select! {
      _ = &mut recv_task => send_task.abort(),
      _ = &mut send_task => recv_task.abort(),

    }
  }

  Ok(())
}

pub async fn http_thread(ip_addr: impl tokio::net::ToSocketAddrs) {
  let listener = TcpListener::bind(ip_addr).await.unwrap();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let io = TokioIo::new(stream);
    tokio::task::spawn(async move {
      if let Err(err) = auto::Builder::new( TokioExecutor::new())
        .serve_connection(io, service_fn(serve))
        .await
      {
        eprintln!("error serving connection: {}", err);
      }
    });
  }
}
