use anyhow::Result;
use futures::future;
use reqwest;
use std::collections::HashMap;

async fn get_page_size(url: &str) -> Result<usize> {
    let response = reqwest::get(url).await?;
    Ok(response.text().await?.len())
}

#[tokio::main]
async fn main() {
    let urls: [&str; 4] = [
        "https://google.com",
        "https://httpbin.org/ip",
        "https://play.rust-lang.org/",
        "BAD_URL",
    ];
    let futures = urls.into_iter().map(get_page_size);
    let results = future::join_all(futures).await;
    let results: HashMap<&str, Result<usize>> = urls.into_iter().zip(results.into_iter()).collect();
    println!("{:?}", results);
}