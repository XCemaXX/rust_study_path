use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_sender: Sender<(SocketAddr, String)>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rec = bcast_sender.subscribe();
    loop {
        tokio::select! {
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if let Some(text_msg) = msg.as_text() {
                            println!("From: {addr:?}. Msg {text_msg}");
                            bcast_sender.send((addr, text_msg.into()))?;
                        } else {
                            println!("Non text msg. From: {addr:?}. {msg:?}")
                        }
                    },
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            },
            msg = bcast_rec.recv() => {
                let (sender_addr, text) = msg?;
                if sender_addr != addr {
                    ws_stream.send(Message::text(text)).await?;
                }
            }, 
            else => return Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_sender, _) = channel(32);

    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("Request on port 2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Request from {addr:?}");
        let bcast_sender = bcast_sender.clone();
        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await?;
            handle_connection(addr, ws_stream, bcast_sender).await
        });
    }
}