
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::{RwLock, broadcast}};
use hyper::server::conn::http1;
use std::sync::Arc;
use hyper::body::Bytes;
use crate::server::handle_request;
use crate::util::ogg_headers::OggHeaders;

use super::TIMEOUT;

pub async fn thread(
  server_addr: impl tokio::net::ToSocketAddrs + std::fmt::Debug + Send + Sync,
  tx: broadcast::Sender<Bytes>,
  header: Arc<RwLock<Option<OggHeaders>>>,
  mount: &'static str,
  allowed_origins: Arc<Option<Vec<&'static str>>>,
  mut shutdown_rx: tokio::sync::watch::Receiver<bool>
) -> anyhow::Result<()> {
  let listener = match TcpListener::bind(&server_addr).await {
    Ok(tl) => tl,
    Err(e) => {
      anyhow::bail!("Could not create TcpListener: {e} {server_addr:#?}");
    }
  };

  'broadcaster: loop {
    tokio::select! {
      _ = shutdown_rx.changed() => { break 'broadcaster }
      Ok((stream, _peer)) = listener.accept() => {
        let _ = stream.set_nodelay(true);
        let io = TokioIo::new(stream);

        tokio::task::spawn({
            let tx = tx.clone();
            let ogg_headers = header.clone();
            let allowed_origins = allowed_origins.clone();
            async move {
              if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(
                move |req| {
                  handle_request(
                    req,
                    tx.clone(),
                    ogg_headers.clone(),
                    mount,
                    allowed_origins.clone()
                  )
                })
              ).await {
                eprintln!("error serving connection: {err}");
              }
            }
          });
      }
      Err(e) = listener.accept() => {
        eprintln!("Accept error: {e}");
        tokio::time::sleep(TIMEOUT).await; // avoid busy loop
      }
    }
  }

  anyhow::Ok(())
}
