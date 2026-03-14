mod responses;

use std::sync::Arc;
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
  allowed_origins: Arc<Option<Vec<&'static str>>>,
) -> Result<Response<BoxBody<Bytes, Infallible>>> {
  let res = match (req.method(), req.uri().path()) {
    (&Method::GET, path) if path == mount.as_ref() => {
      let mut res = stream_response(
        build_stream_body(&tx, ogg_header).await
      ); 
      apply_cors(&req, &mut res, &allowed_origins.as_deref());
      res
    },
    (&Method::GET, "/" | "/index.html") => {
      
      let html = format!(
        "\
          <html>\
          <body>\
          <div>\
          <p>localhost link to audio stream</p>\
          <a href=\"{mount}\">Audio Stream</a>\
          </div>\
          </body>\
          </html>\
          ");
      let body = http_body_util::Full::new(Bytes::from(html)).boxed();
      default_response(body)
    },

    (&Method::OPTIONS, _) => cors_preflight_response(&req, &allowed_origins.as_deref()),
    _ =>  four_oh_four()
  };
  Ok(res)
}
