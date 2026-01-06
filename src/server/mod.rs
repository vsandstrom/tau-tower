use std::sync::Arc;
use std::convert::Infallible;
use hyper::{ 
  Method, 
  Request,
  Response,
  Result,
  StatusCode,
  body::{Bytes, Frame, Incoming}, 
  header::{
    ACCESS_CONTROL_ALLOW_HEADERS,
    ACCESS_CONTROL_ALLOW_ORIGIN,
    ACCESS_CONTROL_EXPOSE_HEADERS,
    ACCESS_CONTROL_MAX_AGE,
    CACHE_CONTROL,
    CONTENT_TYPE,
    ORIGIN,
    VARY
  }
};
use tokio::sync::{RwLock, broadcast};
use futures_util::{Stream, StreamExt, stream};
use http_body_util::{
  BodyExt,
  Empty,
  Full,
  StreamBody,
  combinators::BoxBody
};
use crate::util::ogg_headers::Headers;

type HttpResponse = Response<BoxBody<Bytes, Infallible>>;

const CORS_REQ_SRC: &str = "http://127.0.0.1:4000"; // localhost asciinema

pub async fn handle_request(
    req: Request<Incoming>,
    tx: broadcast::Sender<Bytes>,
    ogg_header: Arc<RwLock<Option<Headers>>>,
    mount: Arc<str>
) -> Result<Response<BoxBody<Bytes, Infallible>>> {
  match (req.method(), req.uri().path()) {
    (&Method::GET, path) if path == &*mount => {
      let rx = tx.subscribe();
      let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(
          |msg| 
          async move { msg.ok() }
        )
        .map(
          |chunk| 
            Ok::<Frame<Bytes>, Infallible>( 
              Frame::data( chunk)
            )
        )
        .take_while(
          |res|
            futures_util::future::ready(res.is_ok())
        );

      // wait for headers to be populated
      let headers = wait_for_ogg_headers(&ogg_header).await;
      // prepend the ogg headers to the stream body
      let stream = prepare_header_stream(headers)
        .chain(stream);

      let body: BoxBody<Bytes, Infallible> = BodyExt::boxed(StreamBody::new(stream));
      let mut res = stream_response(body);
      apply_cors(&req, &mut res);
      Ok(res)
    },

    (&Method::GET, "/" | "/index.html") => {
      let html = format!("<html><body><a href=\"/{mount}\">Audio Stream</a></body></html>");
      let body = http_body_util::Full::new(Bytes::from(html)).boxed();
      let mut res = default_response(body);
      apply_cors(&req, &mut res);
      Ok(res)
    },
    (&Method::OPTIONS, _) => {
      let mut res = cors_preflight_response();
      apply_cors(&req, &mut res);
      Ok(res)
    }
    _ => {
      let mut res = four_oh_four();
      apply_cors(&req, &mut res);
      Ok(res)
    }
  }
}

/// Prevent listeners receiving broken streams, when [`Headers`] struct has not been populated by
/// source stream
async fn wait_for_ogg_headers(header: &Arc<RwLock<Option<Headers>>>) -> Headers {
  loop {
    if let Some(h) = header.read().await.as_ref() {
      return h.clone()
    }
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
  }

}

/// Prepares the Ogg Opus headers, captured from the source stream, to be broadcast on every new
/// listener connection.
fn prepare_header_stream(header: Headers) -> impl Stream<Item = core::result::Result<Frame<Bytes>, Infallible>> {
  stream::iter([Ok(Frame::data(header.head)), Ok(Frame::data(header.tags))].into_iter())
}

/// 
fn stream_response(body: BoxBody<Bytes, Infallible>) -> HttpResponse {
  Response::builder()
  .status(StatusCode::OK)
  .header(CONTENT_TYPE, "audio/ogg; codecs=\"opus\"")
  .header(CACHE_CONTROL, "no_cache")
  // .header(TRANSFER_ENCODING, "chunked")
  // .header(CONNECTION, "keep-alive")
  .header(ACCESS_CONTROL_ALLOW_ORIGIN, CORS_REQ_SRC)
  .body(body)
  .expect("Could not build a stream_response")
}

fn cors_preflight_response() -> HttpResponse {
  Response::builder()
    .status(StatusCode::NO_CONTENT)
    .header(ACCESS_CONTROL_ALLOW_ORIGIN, CORS_REQ_SRC)
    .header(ACCESS_CONTROL_ALLOW_HEADERS,"GET, OPTIONS")
    .header(ACCESS_CONTROL_EXPOSE_HEADERS, "Content-Type, Authorization")
    .header(ACCESS_CONTROL_MAX_AGE, "86400")
    .body(BoxBody::new(Empty::<Bytes>::new()))
    .expect("could not build cors_preflight_response")
}

fn apply_cors(req: &Request<Incoming>, res: &mut HttpResponse) {
  if let Some(origin) = req.headers().get(ORIGIN) {
    res.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
    res.headers_mut().append(VARY, "Origin".parse().expect("could not parse value string"));
  }
}

fn default_response(body: BoxBody<Bytes, Infallible>) -> HttpResponse {
  Response::builder()
  .status(StatusCode::OK)
  .header(CONTENT_TYPE, "text/html; charset=utf-8")
  .header(ACCESS_CONTROL_ALLOW_ORIGIN, CORS_REQ_SRC)
  .body(body)
  .expect("could not build default_response")
}

fn four_oh_four() -> HttpResponse {
  Response::builder()
    .status(StatusCode::NOT_FOUND)
    .body(
      Full::new("NOT_FOUND".into())
        .map_err(|e| match e {})
        .boxed(),
    )
  .expect("could not build 404 response")
}
