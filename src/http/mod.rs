use futures_util::stream::TryStreamExt;
use http_body_util::{BodyExt, Full, StreamBody, combinators::BoxBody};
use hyper::{
  Method, Request, Response, Result, StatusCode,
  body::{Bytes, Frame},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub async fn serve(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
  match *req.method() {
    Method::GET => {
      // Extract path from URI
      let mut path = req.uri().path().to_string();

      // Default to index.html
      if path == "/" {
        path = "/index.html".into();
      }

      // prevent directory traversal like "../.."
      if path.contains("..") {
        return Ok(not_found());
      }

      // request path to file in client/ folder
      let file_path = format!("client/{}", path);
      send_file(&file_path).await
    },
    Method::POST => { todo!() },
    _ => Ok(not_found()),
  }
}

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
