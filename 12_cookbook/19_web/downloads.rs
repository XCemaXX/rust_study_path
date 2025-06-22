use std::{
    error::Error,
    fs::{self, File},
    io::Write,
};

use reqwest::header;
use tempfile::Builder;

async fn download(target: &str) -> Result<(), Box<dyn Error>> {
    let tmp_dir = Builder::new().prefix("download").tempdir()?;
    let res = reqwest::get(target).await?;
    let content_length = res
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let name = res
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.deb");
    let path = tmp_dir.path().join(name);
    println!(
        "Download file `{}` with size {} bytes",
        path.display(),
        content_length
    );
    let mut dest = File::create(&path)?;
    let content = res.bytes().await?;
    dest.write_all(&content)?;
    drop(dest);

    let metadata = fs::metadata(&path)?;
    println!("Downloaded file size on fs: {} bytes", metadata.len());
    assert_eq!(metadata.len(), content_length);
    Ok(())
}

async fn paste() -> Result<(), Box<dyn Error>> {
    let paste_api = "https://paste.rs";
    let contents = r#"fn main() { println!("hello world!");}"#;

    let client = reqwest::Client::new();
    let res = client.post(paste_api).body(contents).send().await?;
    let response = res.text().await?;
    println!("Check paste at: {response}");
    Ok(())
}

mod partial_download {
    use std::{
        error::Error,
        fs::{self, File},
        io::Write,
    };

    use reqwest::{
        StatusCode,
        header::{CONTENT_LENGTH, HeaderValue, RANGE},
    };
    use tempfile::Builder;

    struct PartialRangeIter {
        start: u64,
        end: u64,
        buffer_size: u32,
    }

    impl PartialRangeIter {
        fn new(end: u64, buffer_size: u32) -> Result<Self, &'static str> {
            if buffer_size == 0 {
                Err("Invalid buffer size")?;
            }
            Ok(Self {
                start: 0,
                end,
                buffer_size,
            })
        }
    }

    impl Iterator for PartialRangeIter {
        type Item = HeaderValue;

        fn next(&mut self) -> Option<Self::Item> {
            if self.start > self.end {
                None
            } else {
                let prev_start = self.start;
                self.start += (self.end + 1 - self.start).min(self.buffer_size as u64);
                Some(
                    HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1))
                        .expect("string provided by format!"),
                )
            }
        }
    }

    pub async fn download_by_parts(target: &str) -> Result<(), Box<dyn Error>> {
        const CHUNK_SIZE: u32 = 262_144;
        let client = reqwest::Client::new();
        let res = client.head(target).send().await?;
        let size = res
            .headers()
            .get(CONTENT_LENGTH)
            .ok_or("response doesn't include the content length")?
            .to_str()?
            .parse::<u64>()?;

        let tmp_dir = Builder::new().prefix("download_partly").tempdir()?;
        let path = tmp_dir.path().join("some.deb");
        let mut dest = File::create(&path)?;

        println!("Downloading...");
        for range in PartialRangeIter::new(size - 1, CHUNK_SIZE)? {
            println!("range: {range:?}");
            let mut res = client.get(target).header(RANGE, range).send().await?;
            let status = res.status();
            if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
                Err(format!("Unexpected server response: {status}"))?
            }
            while let Some(chunk) = res.chunk().await? {
                dest.write_all(&chunk)?;
            }
        }
        drop(dest);

        let metadata = fs::metadata(&path)?;
        println!(
            "Partly downloaded file size on fs: {} bytes",
            metadata.len()
        );
        assert_eq!(metadata.len(), size);

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let target = "https://archive.ubuntu.com/ubuntu/ubuntu/pool/main/l/linux/linux-headers-4.15.0-184-generic_4.15.0-184.194_amd64.deb";
    download(target).await.unwrap();
    paste().await.unwrap();
    partial_download::download_by_parts(target).await.unwrap();
}
