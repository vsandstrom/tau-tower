use futures_util::{StreamExt, Stream};
use futures_util::TryStreamExt;
use http_body_util::{Full, Empty, StreamBody, BodyExt, combinators::{BoxBody}};
use hyper::{ 
  Method, Request, Response, Result, StatusCode, 
  body::{Bytes, Incoming, Frame},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tokio::sync::broadcast;

use std::sync::{Arc, Mutex};
use std::convert::Infallible;

struct BroadcastStream { rx: broadcast::Receiver<Vec<u8>> }

impl Stream for BroadcastStream {
  type Item = std::result::Result<Bytes, std::io::Error>;
  fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
    match self.rx.try_recv() {
      Ok(data) => std::task::Poll::Ready(Some(Ok(Bytes::from(data)))),
      Err(broadcast::error::TryRecvError::Empty) => {
        cx.waker().wake_by_ref();
        std::task::Poll::Pending
      },
      Err(broadcast::error::TryRecvError::Lagged(_)) => {
        cx.waker().wake_by_ref();
        std::task::Poll::Pending
      },
      Err(broadcast::error::TryRecvError::Closed) => {std::task::Poll::Ready(None)}
    }
  }
}

// async fn serve_audio(
//   _req: Request<StreamBody<BroadcastStream>>,
//   tx: broadcast::Sender<Vec<u8>>
// ) -> std::result::Result<Response<StreamBody<BroadcastStream>>, Infallible> {
// }

// pub async fn serve(
//     req: Request<hyper::body::Incoming>,
//     tx: broadcast::Sender<Vec<u8>>
// ) -> std::result::Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::Error>
// {
//   match (req.method(), req.uri().path()) {
//     (&Method::GET, "/tau.ogg") => {
//       let rx = tx.subscribe();
//
//       let stream = TokioBroadcastStream::new(rx).map(|res| -> Result<Bytes, std::io::Error> {
//           match res {
//             Ok(vec) => Ok(Bytes::from(vec)),
//             Err(BroadcastStreamRecvError::Lagged(_)) => Err(std::io::Error::new(std::io::ErrorKind::Other, "lagg")),
//           }
//         });
//
//       let body = StreamBody::from(stream);
//       let body = BoxBody::new(body);
//
//       let response = Response::builder()
//         .header("Content-Type", "audio/ogg; codecs=opus")
//         .body(body)
//         .unwrap();
//       Ok(response)
//     },
//     _ => {
//       let not_found_body = futures_util::stream::once( async { 
//         Ok::<_, std::io::Error>(Bytes::from_static(b"Not Found")) 
//       });
//       let body = BoxBody::new(
//         StreamBody::new(
//           not_found_body
//       ));
//       Ok(Response::builder().status(404).body(not_found_body).unwrap())
//       
//     }
//   }
// }

async fn send_file(filename: &str) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
  let file = File::open(filename).await;
  if file.is_err() {
    eprintln!("ERROR: Unable to open file: {filename} {:?}", file);
    return Ok(not_found());
  }
  let file = file.unwrap();

  let mime_type = mime_guess::from_path(filename).first_or_octet_stream();

  let reader_stream = ReaderStream::new(file);
  let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
  let boxed_body = http_body_util::BodyExt::boxed(stream_body);

  let response = Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", mime_type.as_ref()) // <-- Add MIME type
    .body(boxed_body)
    .unwrap();
  Ok(response)
}

fn not_found() -> Response<BoxBody<Bytes, std::io::Error>> {
  Response::builder()
    .status(StatusCode::NOT_FOUND)
    .body(
      Full::new("NOT_FOUND".into())
        .map_err(|e| match e {})
        .boxed(),
    )
  .unwrap()
}

pub async fn handle_request(
    req: Request<Incoming>,
    tx: broadcast::Sender<Vec<u8>>,
    ogg_header: Arc<Mutex<crate::Headers>>
) -> Result<Response<BoxBody<Bytes, Infallible>>> {
  match (req.method(), req.uri().path()) {
    (&Method::GET, "/tau.ogg") => {

    let rx = tx.subscribe();

    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
    .filter_map(|msg| async move { msg.ok() })
    .map(|chunk| Ok::<Frame<Bytes>, Infallible>( Frame::data( Bytes::from(chunk))))
    .take_while(|res| futures_util::future::ready(res.is_ok()));

    let headers = match ogg_header.lock() {
      Ok(h) => {
        if let Some(headers) = &h.headers {
          headers.iter().map(|h| Ok::<Frame<Bytes>,Infallible>(Frame::data(Bytes::from(h.clone())))).collect()
        } else {
          vec!()
        }
      },
      Err(err) => {
        panic!("could not lock headers: {err:?}");
      }
    };

    let stream = futures_util::stream::iter(headers).chain(stream);

    let body: BoxBody<Bytes, Infallible> = BodyExt::boxed(StreamBody::new(stream));
    Ok(Response::builder()
      .status(StatusCode::OK)
      .header("Content-Type", "audio/ogg; codecs=\"opus\"")
      .header("Transfer-Encoding", "chunked")
      .header("Cache-Control", "no-cache")
      .header("Connection", "keep-alive")
      .body(body)
      .unwrap())

    },
    _ => {
      let html = b"<html><body><a href=\"/tau.ogg\">Audio Stream</a></body></html>";
      let body = http_body_util::Full::new(Bytes::from_static(html)).boxed();
      Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(body)
        .unwrap())
    }
  }
}
