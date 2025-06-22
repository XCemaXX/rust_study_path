use std::{
    io::{Error, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Sender},
    thread,
};

fn listen_thread(tx: Sender<SocketAddr>) -> Result<(), Error> {
    let listener = TcpListener::bind("localhost:0")?;
    let port = listener.local_addr()?;
    println!("Listening on port {port}");
    let _ = tx.send(port);
    drop(tx);
    let (mut tcp_stream, addr) = listener.accept()?; // blocks
    let mut input = String::new();
    let _ = tcp_stream.read_to_string(&mut input)?;
    println!("{addr:?} says {input}");
    Ok(())
}

fn main() -> Result<(), Error> {
    let (tx, rx) = mpsc::channel();
    let thread = thread::spawn(move || {
        listen_thread(tx).unwrap();
    });
    let port = rx.recv().unwrap();
    let mut stream = TcpStream::connect(port)?;
    stream.write_all(b"Hello from client")?;
    stream.shutdown(std::net::Shutdown::Write)?;
    thread.join().unwrap();
    Ok(())
}
