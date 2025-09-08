use futures_util::{StreamExt, Stream};
use http_body_util::{Full, StreamBody, BodyExt, combinators::{BoxBody}};
use hyper::{ 
  body::{Bytes, Frame, Incoming}, Method, Request, Response, Result, StatusCode, Uri
};
use tokio::sync::broadcast;

use std::sync::{Arc, Mutex};
use std::convert::Infallible;

fn four_oh_four() -> Response<BoxBody<Bytes, Infallible>> {
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
    tx: broadcast::Sender<Bytes>,
    ogg_header: Arc<Mutex<crate::Headers>>,
    mount: Arc<str>
) -> Result<Response<BoxBody<Bytes, Infallible>>> {
  match (req.method(), req.uri().path()) {
    (&Method::GET, path) if path == &*mount => {
      let rx = tx.subscribe();
      let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(|msg| async move { msg.ok() })
        .map(|chunk| Ok::<Frame<Bytes>, Infallible>( Frame::data( chunk)))
        .take_while(|res| futures_util::future::ready(res.is_ok()));

      let stream = ogg_header_stream(ogg_header).chain(stream);
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

    (&Method::GET, "/" | "/index.html") => {
      let html = format!("<html><body><a href=\"/{mount}\">Audio Stream</a></body></html>");
      let body = http_body_util::Full::new(Bytes::from(html)).boxed();
      Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(body)
        .unwrap())
    },

    _ => {
      Ok(four_oh_four())
    }
  }
}

fn ogg_header_stream(header: Arc<Mutex<crate::Headers>>) -> impl Stream<Item = core::result::Result<Frame<Bytes>, Infallible>> {
  futures_util::stream::iter(
    header
      .lock()
      .ok()
      .and_then(|h| h.headers.clone()).into_iter()
      .map(|b| Ok(Frame::data(b)))
  )
}
