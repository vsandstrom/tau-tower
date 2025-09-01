#![deny(unused_crate_dependencies)]
mod http;
mod threads;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::task;

use crate::http::serve;
use crate::threads::{udp_thread, ws_thread};

const UDP: u16 = 8001;
const PORT: u16 = 8002;
const SOCKET: u16 = 9001;

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
async fn main() -> anyhow::Result<()> {
    let [udp_addr, ip_addr, socket_addr] = [UDP, PORT, SOCKET]
        .map(|port| SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port));

    let (tx, _) = broadcast::channel::<Vec<u8>>(1024);
    let tx_clone = tx.clone();

    // handle udp listener
    task::spawn(async move {
        udp_thread(tx_clone, udp_addr).await.unwrap();
    });

    // handle websocket thread
    task::spawn(async move {
        ws_thread(tx, socket_addr).await.unwrap();
    });

    // handle http serve
    task::spawn(async move {
        let listener = TcpListener::bind(ip_addr).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(serve))
                    .await
                {
                    eprintln!("error serving connection: {}", err);
                }
            });
        }
    });

    futures_util::future::pending::<()>().await;
    Ok(())
}
