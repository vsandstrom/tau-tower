mod responses;

use std::{net::Ipv4Addr, sync::Arc};
use std::convert::Infallible;
use tokio::sync::{RwLock, broadcast};
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::{ 
  Method, 
  Request,
  Response,
  Result,
  body::{Bytes, Incoming}, 
};

use crate::util::ogg_headers::Headers;
use responses::{
  build_stream_body,
  default_response,
  stream_response,
  four_oh_four,
  apply_cors,
  cors_preflight_response
};

pub async fn handle_request(
  req: Request<Incoming>,
  tx: broadcast::Sender<Bytes>,
  ogg_header: Arc<RwLock<Option<Headers>>>,
  mount: Arc<String>,
  allowed_origin: Option<&str>,
) -> Result<Response<BoxBody<Bytes, Infallible>>> {
  let mut res = match (req.method(), req.uri().path()) {
    (&Method::GET, path) if path == mount.as_ref() => 
      stream_response(build_stream_body(&tx, ogg_header).await),

    (&Method::GET, "/" | "/index.html") => {
      let html = format!(
        "<html><body><a href=\"{}{mount}\">Audio Stream</a></body></html>",
        std::net::IpAddr::V4(Ipv4Addr::LOCALHOST)
      );
      let body = http_body_util::Full::new(Bytes::from(html)).boxed();
      default_response(body)
    },

    (&Method::OPTIONS, _) => cors_preflight_response(allowed_origin),
    _ =>  four_oh_four()
  };
  apply_cors(&req, &mut res, allowed_origin);
  Ok(res)
}
