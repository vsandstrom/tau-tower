use hyper::{Request, Response, StatusCode};
use tokio::sync::broadcast;
use std::sync::Arc;

use hyper::body::Bytes;
use tokio_tungstenite::{accept_hdr_async, WebSocketStream};
use tokio::sync::RwLock;
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};
use tokio::time::Instant;
use crate::threads::LOG_TIMEOUT;
use crate::util::credentials::Credentials;
use crate::util::ogg_headers::{OggHeaderType, OggHeaders, parse_ogg_headers};

const TIMEOUT: Duration = Duration::from_millis(50);

/// Creates a WebSocket receiver listening to the sender of the ogg opus stream.
/// Appending the ogg opus blocks to a producer/consumer object.
pub async fn thread(
  tx: broadcast::Sender<Bytes>,
  listen_addr: SocketAddr,
  credentials: Credentials,
  header: Arc<RwLock<Option<OggHeaders>>>,
  mut shutdown_rx: tokio::sync::watch::Receiver<bool>
) -> anyhow::Result<()> {
  let server = match TcpListener::bind(listen_addr).await {
    Ok(s) => s,
    Err(e) => {
      anyhow::bail!("Could not bind to source address: {e}");
    }
  };

  'listener: loop {
    tokio::select! {
      _ = shutdown_rx.changed() => { break 'listener } 
      Ok((stream, addr)) = server.accept() => {
        match accept_hdr_async(stream, |req: &Request<_>, res: hyper::Response<()>| {

          // unbox large error
          validate_headers(req, res, &credentials)
        }
        ).await {
          Ok(mut ws_stream) => {
            receive_data(&mut ws_stream, header.clone(), tx.clone()).await;
          },
          Err(e) => {
            eprintln!("Handshake failed from {addr}: {e}");
          }
        }
      }
      Err(e) = server.accept() => {
        eprintln!("{e}");
        tokio::time::sleep(TIMEOUT).await;
      }
    }
  }

  anyhow::Ok(())
}


#[allow(clippy::result_large_err)]
fn validate_headers(req: &Request<()>, res: hyper::Response<()>, credentials: &Credentials) -> Result<Response<()>, Response<Option<String>>> {
  let (Some(username), Some(password)) = (
    req.headers().get("username").and_then(|u| u.to_str().ok()),
    req.headers().get("password").and_then(|p| p.to_str().ok())
  ) else {
    let mut res = Response::new(Some("Unauthorized access: 401".to_string()));
    *res.status_mut() = StatusCode::UNAUTHORIZED;
    return Err(res);
  };

  if !credentials.validate(username, password) {
    let mut res = Response::new(Some("Access forbidden: 403".to_string()));
    *res.status_mut() = StatusCode::FORBIDDEN;
    return Err(res);
  }

  Ok(res)
}

async fn receive_data(ws_stream: &mut WebSocketStream<TcpStream>, header: Arc<RwLock<Option<OggHeaders>>>, tx: broadcast::Sender<Bytes>) {
  let mut temp_headers: (Option<Bytes>, Option<Bytes>) = (None, None);
  let mut headers_parsed = false;
  let mut last_log = Instant::now();
  'connections: while let Some(msg) = ws_stream.next().await  {
    let page = match msg {
      Ok(m) => m.into_data(),
      Err(e) => {
        eprintln!("Unrecognized message: {e}");
        break 'connections;
      }
    };

    // short circuit if headers have already been parsed.
    if !headers_parsed { 
      match parse_ogg_headers(&page) {
        OggHeaderType::Head(head) => {temp_headers.0 = Some(head);},
        OggHeaderType::Tags(tags) => {temp_headers.1 = Some(tags);},
        OggHeaderType::None => {}
      }
      
      if let (Some(head), Some(tags)) = &temp_headers 
        && let Ok(mut h) = header.try_write() 
        && (*h).is_none() {
        *h = Some(OggHeaders::new((head.clone(), tags.clone())));
        headers_parsed = true;
      }
    }

    if let Err(e) = tx.send(page) 
      && last_log.elapsed() > LOG_TIMEOUT {
      eprintln!("could not open client stream: {e}"); 
      // Flushing headers if connection is lost
      temp_headers = (None, None);
      headers_parsed = false;
      last_log = Instant::now();
    }
  }
}
