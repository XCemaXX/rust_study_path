use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    let (mut ws_stream, _) = ClientBuilder::from_uri(
        Uri::from_static("ws://127.0.0.1:2000")
    ).connect().await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            Ok(Some(line)) = stdin.next_line() => {
                let msg = Message::text(line);
                ws_stream.send(msg).await?;
            },
            Some(Ok(msg)) = ws_stream.next() => {
                if let Some(text_msg) = msg.as_text() {
                    println!("From server: {}", text_msg);
                } else {
                    println!("Non text msg from server: {:?}", msg);
                }
            },
            else => return Ok(()),
        }
    }
}