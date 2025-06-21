use std::error::Error;

use url::{Origin, ParseError, Position, Url};

fn url_from_str(s: &str) -> Result<(), ParseError> {
    let parsed = Url::parse(s)?;
    println!("The path part of the URL is: {}", parsed.path());
    Ok(())
}

fn base_url(s: &str) -> Result<Url, Box<dyn Error>> {
    let mut url = Url::parse(s)?;
    let base = {
        match url.path_segments_mut() {
            Ok(mut path) => path.clear(),
            Err(_) => return Err("Cannot be a base".into()),
        };
        url.set_query(None);
        url.set_fragment(None);
        url
    };
    Ok(base)
}

fn build_new_url(s: &str, path: &str) -> Result<(), Box<dyn Error>> {
    let base = base_url(s)?;
    let joined = base.join(path)?;
    println!("Joined URL is: {joined}");
    Ok(())
}

fn extract_origin(s: &str) -> Result<(), Box<dyn Error>> {
    let url = Url::parse(s)?;
    let scheme = url.scheme();
    let host = url.host().ok_or("Host is None")?.to_owned();
    let port = url.port();
    println!("Scheme: {}, host: {:?}, port: {:?}", scheme, host, port);
    let port = url.port().unwrap_or(255);
    let combined = Origin::Tuple(scheme.to_owned(), host, port);
    println!("Origin: {:?}", combined);
    Ok(())
}

fn clean(s: &str) -> Result<(), Box<dyn Error>> {
    let url = Url::parse(s)?;
    let cleaned = &url[..Position::AfterPath];
    println!("Cleaned: {cleaned}");
    Ok(())
}

fn main() {
    let s = "https://rust-lang-nursery.github.io/rust-cookbook/web/url.html#parse-a-url-from-a-string-to-a-url-type";
    println!("URL: {s}");
    url_from_str(s).unwrap();
    let base = base_url(s).unwrap();
    println!("The base of the URL is: {base}");
    build_new_url(s, "/rust-lang/cargo").unwrap();
    extract_origin(s).unwrap();
    clean(s).unwrap();
}
