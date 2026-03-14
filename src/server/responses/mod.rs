use std::sync::Arc;
use futures_util::StreamExt;
use http_body_util::StreamBody;
use std::convert::Infallible;
use hyper::{ 
  StatusCode,
  Request,
  Response,
  body::{Bytes, Frame, Incoming}, 
  header::{
    ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS, ACCESS_CONTROL_MAX_AGE, CACHE_CONTROL, CONTENT_TYPE, HeaderValue, ORIGIN, VARY
  }
};
use tokio::sync::{RwLock, broadcast};
use futures_util::{Stream, stream};
use http_body_util::{
  BodyExt,
  Empty,
  Full,
  combinators::BoxBody
};
use crate::util::ogg_headers::Headers;

type HttpResponse = Response<BoxBody<Bytes, Infallible>>;


/// Builds the HTTP audio stream from Tokio BroadcastStream.
/// It waits for the headers of the OggOpus stream to be available and takes care of prepending
/// them to each new consumer stream. 
pub(super) async fn build_stream_body(
  tx: &broadcast::Sender<Bytes>, 
  ogg_header: Arc<RwLock<Option<Headers>>>) 
-> BoxBody<Bytes, Infallible> {
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

  BodyExt::boxed(StreamBody::new(stream))
}

/// Prepares the Ogg Opus headers, captured from the source stream, to be broadcast on every new
/// listener connection.
pub(super) fn prepare_header_stream(header: Headers) -> impl Stream<Item = core::result::Result<Frame<Bytes>, Infallible>> {
  stream::iter([Ok(Frame::data(header.head)), Ok(Frame::data(header.tags))])
}


/// Prevent listeners receiving broken streams, when [`Headers`] struct has not been populated by
/// source stream
pub(super) async fn wait_for_ogg_headers(header: &Arc<RwLock<Option<Headers>>>) -> Headers {
  loop {
    if let Some(h) = header.read().await.as_ref() {
      return h.clone()
    }
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
  }
}

pub(super) fn stream_response(body: BoxBody<Bytes, Infallible>) -> HttpResponse {
  match Response::builder()
  .status(StatusCode::OK)
  .header(CONTENT_TYPE, "audio/ogg; codecs=\"opus\"")
  .header(CACHE_CONTROL, "no-cache")
  .body(body) {
    Ok(res) => res,
    Err(e) => unreachable!("Could not build a stream_response: {e}")
  }

}

pub(super) fn cors_preflight_response(
  req: &Request<Incoming>,
  allowed_origin: &Option<&[&'static str]>) -> HttpResponse {
  let forbidden = || match Response::builder()
    .status(StatusCode::FORBIDDEN)
    .body(BoxBody::new(Empty::<Bytes>::new())) {
      Ok(res) => res,
      Err(e) => unreachable!("unable to build cors_preflight_response: {e}")
    };

  let Some(origins) = allowed_origin else {
    return forbidden();
  };

  let Some(request_origin) = req.headers().get(ORIGIN) else {
    return forbidden();
  };
  
  let allowed = match origins {
    ["*"] => "*", 
    _ => match origins.iter().find(|&&o| o.as_bytes() == request_origin.as_bytes()) {
      Some(&o) => o,
      None => return forbidden(),
    }
  };

  match Response::builder() 
    .status(StatusCode::NO_CONTENT)
    .header(ACCESS_CONTROL_ALLOW_ORIGIN, allowed)
    .header(ACCESS_CONTROL_ALLOW_METHODS,"GET, OPTIONS")
    .header(ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization")
    .header(ACCESS_CONTROL_EXPOSE_HEADERS,"Content-Type")
    .header(ACCESS_CONTROL_MAX_AGE, "86400")
    .body(BoxBody::new(Empty::<Bytes>::new())) {
      Ok(res) => res,
      Err(e) => unreachable!("unable to build cors_preflight_response: {e}")
  }
}

pub(super) fn apply_cors(
  req: &Request<Incoming>, 
  res: &mut HttpResponse, 
  allowed_origins: &Option<&[&'static str]>
) {
  let Some(origins) = allowed_origins else { return; };
  let Some(request_origin) = req.headers().get(ORIGIN) else { return; };

  let allowed = match origins {
    ["*"] => "*", 
    _ => match origins.iter().find(|&&o| o.as_bytes() == request_origin.as_bytes()) {
      Some(&o) => o,
      None => return
    }
  };

  res.headers_mut().insert( ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static(allowed));
  res.headers_mut().append( VARY, HeaderValue::from_static("Origin"));
}

pub(super) fn default_response(body: BoxBody<Bytes, Infallible>) -> HttpResponse {
  match Response::builder()
  .status(StatusCode::OK)
  .header(CONTENT_TYPE, "text/html; charset=utf-8")
  .body(body) {
    Ok(res) => res,
    Err(e) => unreachable!("unable to build default_response: {e}")
  }
}

pub(super) fn four_oh_four() -> HttpResponse {
  match Response::builder()
    .status(StatusCode::NOT_FOUND)
    .body(
      Full::new("NOT_FOUND".into())
        .map_err(|e| match e {})
        .boxed(),
  ) {
    Ok(res) => res,
    Err(e) => unreachable!("unable to build 404 response: {e}")
  }
}
