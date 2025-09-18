use hyper::Uri;
use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};

use hyper::body::Bytes;
use tokio_tungstenite::connect_async;
use futures_util::StreamExt;
use std::net::SocketAddr;
use tokio_tungstenite::tungstenite::ClientRequestBuilder;
use std::time::Duration;

use super::{prepare_headers, Headers, validate_bos_and_tags};
use crate::Credentials;

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
  let uri = Uri::builder()
    .scheme("ws")
    .authority(format!("{}:{}", src_addr.ip(), src_addr.port()))
    .path_and_query("/")
    .build()
    .unwrap();
  // let uri = Uri::from_static(&url);
  let mut temp_headers: Vec<Bytes> = vec!();
  let mut headers_parsed = false;
  let mut open_endpoint = true;
  loop {
    let request = ClientRequestBuilder::new(uri.clone())
      .with_header("port", credentials.broadcast_port.to_string())
      .with_header("password", credentials.password.clone())
      .with_header("username", credentials.username.clone());
    match connect_async(request).await {
      Ok((mut ws_stream, _)) => {
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
              h.headers = Some(prepare_headers(&temp_headers));
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
        // eprintln!("WS connect failed {e}");
        tokio::time::sleep(TIMEOUT).await;
      }
    }
  }
}
