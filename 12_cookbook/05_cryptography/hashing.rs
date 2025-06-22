use std::io::{BufReader, Read};

use ring::{
    digest::{Context, Digest, SHA256},
    hmac,
    rand::{self, SecureRandom},
};

fn sha256_digest<R: Read>(mut reader: R) -> std::io::Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    Ok(context.finish())
}

fn calc256() {
    let buf = "We will generate a digest of this text";
    let reader = BufReader::new(buf.as_bytes());
    let digest = sha256_digest(reader).expect("Should calc");

    println!("Digest is {:?}", digest);
}

fn verify_msg() {
    let mut key_value = [0u8; 48];
    let rng = rand::SystemRandom::new();
    rng.fill(&mut key_value).expect("Rng problem");
    let key = hmac::Key::new(hmac::HMAC_SHA256, &key_value);

    let msg = "Some message to verify";
    let signature = hmac::sign(&key, msg.as_bytes());

    println!("Signature: {:?}", signature);
    let should_succeed = hmac::verify(&key, msg.as_bytes(), signature.as_ref());
    assert!(should_succeed.is_ok());
}

fn main() {
    calc256();
    verify_msg();
}
