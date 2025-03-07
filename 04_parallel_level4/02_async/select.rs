use tokio::sync::mpsc::{self, Receiver};
use tokio::time::{sleep, Duration};

#[derive(Debug, PartialEq)]
enum Packet {
    Tcp { data: String },
    Udp { data: String },
}

async fn get_fastest(
    mut tcp_rcv: Receiver<String>,
    mut udp_rcv: Receiver<String>,
) -> Option<Packet> {
    tokio::select! {
        tcp_data = tcp_rcv.recv() => Some(Packet::Tcp {data: tcp_data? }),
        udp_data = udp_rcv.recv() => Some(Packet::Udp {data: udp_data? }),
    }
}

#[tokio::main]
async fn main() {
    let tcp_channel = mpsc::channel(32);
    let udp_channel = mpsc::channel(32);

    tokio::spawn(async move {
        println!("tcp start");
        sleep(Duration::from_millis(500)).await;
        tcp_channel.0.send("tcp_data".to_string()).await.expect("send err");
        println!("tcp end");
    });
    tokio::spawn(async move {
        println!("udp start");
        sleep(Duration::from_millis(50)).await;
        udp_channel.0.send("udp_data".to_string()).await.expect("send err");
        println!("udp end");
    });
    println!("lets go");
    let fastest = get_fastest(tcp_channel.1, udp_channel.1).await
        .expect("err");
    println!("Fastest {fastest:?}");
}