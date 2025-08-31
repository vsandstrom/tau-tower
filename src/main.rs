use std::net::{UdpSocket};
use crossbeam::channel::bounded;
use std::thread::spawn;
use hyper_util::rt::TokioIo;
use hyper::service::service_fn;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::Frame;
use hyper::{Method, Request, Response, Result, StatusCode};
use tokio::{fs::File, net::TcpListener};
use tokio_util::io::ReaderStream;
use futures_util::stream::TryStreamExt;
use hyper::body::Body;
use mime_guess;


const UDP: u16 = 8001;
const PORT: u16 = 8002;
const IP: &str = "127.0.0.1";

const INDEX: &str = "client/index.html";

#[derive(Clone)]
// An Executor that uses the tokio runtime.
pub struct TokioExecutor;

// Implement the `hyper::rt::Executor` trait for `TokioExecutor` so that it can be used to spawn
// tasks in the hyper runtime.
// An Executor allows us to manage execution of tasks which can help us improve the efficiency and
// scalability of the server.
impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}


#[tokio::main]
async fn main() -> Result<()> {
  let (tx, rx) = bounded::<Vec<u8>>(1024);
  let receiver_thread = spawn(move || {
    let socket = match UdpSocket::bind(format!("127.0.0.1:{PORT}")) {
      Ok(s) => s,
      Err(e) => {eprintln!("Udp Socket could not connect to address: {e}"); std::process::exit(1)}
    };

    spawn(move || {
      let mut buf = [0; 256];
      while let Ok(len) = socket.recv(&mut buf) { 
        // println!("{:?}", &buf[..len]);
        if let Err(e) = tx.send(buf[..len].as_ref().to_vec()) {
          panic!("{e:?}");
        }
      }
    });
  });

  let listener = TcpListener::bind(format!("{IP}:{PORT}")).await.unwrap();

  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let io = TokioIo::new(stream);
    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new()
        .serve_connection(io, service_fn(response_examples))
        .await {
        eprintln!("error serving connection: {}", err);
      }
    });
  }
  Ok(())
}

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

async fn response_examples(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    if req.method() != Method::GET {
        return Ok(not_found());
    }

    // Extract path from URI
    let mut path = req.uri().path().to_string();

    // Default to index.html if root is requested
    if path == "/" {
        path = "/index.html".into();
    }

    // Security: prevent directory traversal like "../../secret"
    if path.contains("..") {
        return Ok(not_found());
    }

    // Map request path to file in the client/ folder
    let file_path = format!("client{}", path);

    simple_file_send(&file_path).await
}

async fn simple_file_send(filename: &str) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    let file = File::open(filename).await;
    if file.is_err() {
        eprintln!("ERROR: Unable to open file.");
        return Ok(not_found());
    }
    let file = file.unwrap();

    let mime_type = mime_guess::from_path(filename).first_or_octet_stream();

    let reader_stream = ReaderStream::new(file);
    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
    let boxed_body = stream_body.boxed();

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
        .body(Full::new("NOT_FOUND".into()).map_err(|e| match e {}).boxed())
        .unwrap()
}
