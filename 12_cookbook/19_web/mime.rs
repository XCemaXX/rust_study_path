use std::{error::Error, str::FromStr};

use mime::{APPLICATION_OCTET_STREAM, Mime};
use reqwest::header::CONTENT_TYPE;

fn from_str() {
    let invalid_mime_type = "i n v a l i d";
    let parsed_mime = invalid_mime_type
        .parse::<Mime>()
        .unwrap_or(APPLICATION_OCTET_STREAM);
    println!(
        "MIME for `{}` used default value: `{}`",
        invalid_mime_type, parsed_mime
    );

    let valid_mime_type = "TEXT/PLAIN";
    let parsed_mime = valid_mime_type
        .parse::<Mime>()
        .unwrap_or(APPLICATION_OCTET_STREAM);
    println!("MIME for `{}` is: `{}`", valid_mime_type, parsed_mime);
}

fn from_file_name(name: &str) {
    let parts: Vec<_> = name.split('.').collect();

    let r#type = match parts.last() {
        Some(v) => match *v {
            "png" => mime::IMAGE_PNG,
            "jpg" | "jpeg" => mime::IMAGE_JPEG,
            "json" => mime::APPLICATION_JSON,
            _ => mime::TEXT_PLAIN,
        },
        None => mime::TEXT_PLAIN,
    };
    println!("Mime for file `{}` is `{}`", name, r#type);
}

async fn for_http() -> Result<(), Box<dyn Error>> {
    let res = reqwest::get("https://badge-cache.kominick.com/crates/v/csv.svg?label=mime").await?;
    let headers = res.headers();
    match headers.get(CONTENT_TYPE) {
        Some(t) => {
            let content_type = Mime::from_str(t.to_str()?)?;
            let media_type = match (content_type.type_(), content_type.subtype()) {
                (mime::TEXT, mime::HTML) => "a HTML document",
                (mime::TEXT, _) => "a text document",
                (mime::IMAGE, mime::PNG) => "a PNG image",
                (mime::IMAGE, _) => "an image",
                _ => "neither text nor image",
            };
            println!("The reponse contains {}.", media_type);
        }
        None => {
            println!("The response does not contain a Content-Type header.");
        }
    }
    Ok(())
}

fn main() {
    from_str();
    ["foobar.jpg", "foo.bar", "foobar.png"]
        .into_iter()
        .for_each(from_file_name);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(for_http()).unwrap();
}
