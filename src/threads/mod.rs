use futures_util::{SinkExt, StreamExt};
use hyper::Result;
use tokio::{net::TcpListener, sync::broadcast, task};
use tokio_tungstenite::{accept_async, tungstenite::Message};

const MTU: usize = 1500;

pub async fn udp_thread(
    tx: broadcast::Sender<Vec<u8>>,
    udp_addr: impl tokio::net::ToSocketAddrs,
) -> Result<()> {
    use tokio::net::UdpSocket;
    let socket = match UdpSocket::bind(udp_addr).await {
        Ok(s) => s,
        Err(e) => panic!("Could not connect to UDP port: {e}"),
    };

    let mut buf = [0u8; MTU];
    loop {
        while let Ok(size) = socket.recv(&mut buf).await {
            let msg = buf[..size].to_vec();
            tx.send(msg).unwrap();
        }
    }
}

pub async fn ws_thread(
    tx: broadcast::Sender<Vec<u8>>,
    socket_addr: impl tokio::net::ToSocketAddrs,
) -> Result<()> {
    let ws_listener = TcpListener::bind(socket_addr).await.unwrap();
    while let Ok((stream, _)) = ws_listener.accept().await {
        let ws = accept_async(stream).await.unwrap();
        let (mut tx_ws, mut rx_ws) = ws.split();
        let mut rx = tx.subscribe();

        let mut send_task = task::spawn(async move {
            while let Ok(buf) = rx.recv().await {
                let msg = Message::binary(buf);
                let _ = tx_ws.send(msg).await;
            }
        });

        // keeping the websocket alive with fw polling
        let mut recv_task =
            task::spawn(async move { while let Some(Ok(_msg)) = rx_ws.next().await {} });

        tokio::select! {
          _ = &mut recv_task => send_task.abort(),
          _ = &mut send_task => recv_task.abort(),

        }
    }

    Ok(())
}
