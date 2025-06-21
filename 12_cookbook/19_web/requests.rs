use std::{collections::HashMap, error::Error, io::Read};

use reqwest::{
    blocking::Client,
    header::{HeaderMap, USER_AGENT},
};
use serde::Deserialize;
use url::Url;

fn simple() -> Result<(), Box<dyn Error>> {
    let mut res = reqwest::blocking::get(
        "https://rust-lang-nursery.github.io/rust-cookbook/web/clients/requests.html",
    )?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    body = body.chars().take(100).collect();
    let headers: HeaderMap = res
        .headers()
        .iter()
        .take(5)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    println!(
        "Status: {}\nHeaders:\n{:#?}\nBody{}",
        res.status(),
        headers,
        body
    );

    Ok(())
}

async fn simple_async() -> Result<(), Box<dyn Error>> {
    let res =
        reqwest::get("https://rust-lang-nursery.github.io/rust-cookbook/web/clients/requests.html")
            .await?;
    println!("Status async: {}", res.status(),);
    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct HeadersEcho {
    pub headers: HashMap<String, String>,
}

fn complex_request() -> Result<(), Box<dyn Error>> {
    let url = Url::parse_with_params(
        "http://httpbin.org/headers",
        &[("lang", "rust"), ("browser", "servo")],
    )?;
    let res = Client::new()
        .get(url)
        .header(USER_AGENT, "Rust-test-agent")
        .header("X-Powered-By", "Rust")
        .send()?;
    assert_eq!(
        res.url().as_str(),
        "http://httpbin.org/headers?lang=rust&browser=servo"
    );
    let out: HeadersEcho = res.json()?;
    assert_eq!(out.headers["User-Agent"], "Rust-test-agent");
    assert_eq!(out.headers["X-Powered-By"], "Rust");

    Ok(())
}

fn main() {
    simple().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(simple_async()).unwrap();
    complex_request().unwrap();
}
