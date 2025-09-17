
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::broadcast};
use hyper::server::conn::http1;
use std::sync::{Arc, Mutex};
use crate::server::handle_request;
use hyper::body::Bytes;

use super::{TIMEOUT, Headers};

pub async fn thread(
    ip_addr: impl tokio::net::ToSocketAddrs,
    tx: broadcast::Sender<Bytes>,
    header: &Arc<Mutex<Headers>>,
    mount: Arc<str>
) {
  let listener = TcpListener::bind(ip_addr).await.unwrap();
  let tx_clone = tx.clone();
  let mount = mount.clone();

  loop {
    let (stream, _peer) = match listener.accept().await {
      Ok(sp) => sp,
      Err(e) => {
          eprintln!("Accept error: {e}");
          tokio::time::sleep(TIMEOUT).await; // avoid busy loop
          continue;
      }
    };

    let _ = stream.set_nodelay(true);
    let io = TokioIo::new(stream);
    let tx_inner_clone = tx_clone.clone();
    let header_clone = header.clone();
    let mount_clone = mount.clone();
    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new()
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
