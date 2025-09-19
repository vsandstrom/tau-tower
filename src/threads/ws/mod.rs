use hyper::{Request, Response, StatusCode};
use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};

use hyper::body::Bytes;
use tokio_tungstenite::accept_hdr_async;
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::time::Duration;

use crate::util::{Headers, validate_bos_and_tags, Credentials};
use tokio::net::TcpListener;

pub const MTU: usize = 1500;
const TIMEOUT: Duration = Duration::from_millis(50);

/// Creates a WebSocket receiver listening to the sender of the ogg opus stream.
/// Appending the ogg opus blocks to a producer/consumer object.
pub async fn thread(
    tx: broadcast::Sender<Bytes>,
    src_addr: SocketAddr,
    credentials: Credentials,
    header: Arc<Mutex<Headers>>
) {
  // let url = format!("ws://{}:{}", src_addr.ip() , src_addr.port());
  // let uri = Uri::builder()
  //   .scheme("ws")
  //   .authority(format!("{}:{}", src_addr.ip(), src_addr.port()))
  //   .path_and_query("/")
  //   .build()
  //   .unwrap();
  // let uri = Uri::from_static(&url);
  let mut temp_headers: Vec<Bytes> = vec!();
  let mut headers_parsed = false;
  let mut open_endpoint = true;

  let server = match TcpListener::bind(src_addr).await {
    Ok(s) => s,
    Err(e) => {
      eprintln!("Could not bind to source address: {e}");
      return;
    }
  };

  loop {
    match server.accept().await {
      Ok((stream, _)) => {
        match accept_hdr_async(stream, |req: &Request<()>, mut res: hyper::Response<()>| {
          let username = req.headers()
            .get("username")
            .map(|u| u.to_str().unwrap());

          let password = req.headers()
            .get("password")
            .map(|pw| pw.to_str().unwrap());

          let port = req.headers()
            .get("port")
            .map(|port| port.to_str().unwrap().parse::<u16>().unwrap());

          if !credentials.validate(username, password, port) {
            let res = Response::builder()
              .status(StatusCode::FORBIDDEN)
              .body(Some("Credentials do not match".to_string()))
              .unwrap();
            return Err(res);
          }

          *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
          Ok(res)
        }).await {
          Ok(mut ws_stream) => {
            'connections: while let Some(msg) = ws_stream.next().await  {
              let page = match msg {
                Ok(m) => m.into_data(),
                Err(e) => {
                  eprintln!("Unrecognized message: {e}");
                  break 'connections;
                }
              };
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
          },
          Err(e) => {
          }
        }
      },
      Err(e) => {
        // eprintln!("WS connect failed {e}");
        tokio::time::sleep(TIMEOUT).await;
      }
    }
  }
}
