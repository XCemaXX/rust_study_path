use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn tcp_listener_inner(mut socket: tokio::net::TcpStream) -> io::Result<()> {
    socket.write_all(b"Who are you?\n").await?;
    let mut buf = vec![0; 1024];
    let name_size = socket.read(&mut buf).await?;
    let name = std::str::from_utf8(&buf[..name_size]).unwrap().trim();
    let reply = format!("Hi, {name}!\n");
    socket.write_all(reply.as_bytes()).await?;
    Ok(())
}

async fn tcp_listener(socket: tokio::net::TcpStream) {
    if let Err(e) = tcp_listener_inner(socket).await {
        println!("{}", e);
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    println!("Port {}", listener.local_addr()?.port());

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("request from {addr:?}");
        tokio::spawn(tcp_listener(socket));
    }
}

// to connect run in terminal: "nc localhost PORT"