use base64::{Engine, engine::general_purpose};
use data_encoding::HEXUPPER;
use percent_encoding::{AsciiSet, CONTROLS, percent_decode, utf8_percent_encode};
use url::form_urlencoded::{byte_serialize, parse};

fn percent_encode() {
    /// https://url.spec.whatwg.org/#fragment-percent-encode-set
    const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

    let s = "So \"fansy\" string";
    let encoded = utf8_percent_encode(s, FRAGMENT).collect::<String>();
    assert_eq!(encoded, "So%20%22fansy%22%20string");

    let decoded = percent_decode(encoded.as_bytes()).decode_utf8().unwrap();
    assert_eq!(decoded, s);
    println!("decoded:'{}'", decoded);
}

fn url_encode() {
    let s = "What is ❓".as_bytes();
    let encoded = byte_serialize(s).collect::<String>();
    assert_eq!(encoded, "What+is+%E2%9D%93");

    let decoded = parse(encoded.as_bytes())
        .map(|(key, val)| [key, val].concat())
        .collect::<String>();
    assert_eq!(decoded.as_bytes(), s);
    println!("decoded:'{}'", decoded);
}

fn hex_encode() {
    let s = b"The quick brown fox jumps over the lazy dog.";
    let encoded = HEXUPPER.encode(s);
    let expected = "54686520717569636B2062726F776E20666F78206A756D7073206F76\
        657220746865206C617A7920646F672E";
    assert_eq!(encoded, expected);

    let decoded = HEXUPPER.decode(&encoded.as_bytes()).unwrap();
    assert_eq!(decoded, s);
    println!("decoded:'{}'", String::from_utf8(decoded).unwrap());
}

fn base64_encode() {
    let s = b"hello world~";
    let encoded = general_purpose::STANDARD.encode(s);
    let decoded = general_purpose::STANDARD.decode(&encoded).unwrap();
    assert_eq!(decoded, s);
    println!("base64 encoded: {}", encoded);
    println!("decoded:'{}'", String::from_utf8(decoded).unwrap());
}

fn main() {
    percent_encode();
    url_encode();
    hex_encode();
    base64_encode();
}
