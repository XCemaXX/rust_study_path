#![allow(dead_code)]
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;

fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

const PREFIX_IN_TAR: &str = "path/in/tar";

fn compress(tar_path: &Path) -> Result<(), std::io::Error> {
    let tar_gz = File::create(tar_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(PREFIX_IN_TAR, source_dir())?;
    //let enc = tar.into_inner()?;
    //enc.finish()?;
    tar.finish()?;
    let enc = tar.into_inner()?;
    enc.finish()?;
    println!("Packed to: {}", tar_path.display());
    Ok(())
}

fn decompress(tar_path: &Path) -> Result<(), std::io::Error> {
    let output_path = Path::new("/tmp/unpacked");
    std::fs::create_dir_all(output_path)?;
    let tar_gz = File::open(tar_path)?;
    let mut archive = Archive::new(GzDecoder::new(tar_gz));
    //archive.unpack(output_path)?;
    println!("Extracted the following files:");
    archive
        .entries()?
        .filter_map(|e| e.ok())
        .map(|mut entry| -> Result<_, Box<dyn std::error::Error>> {
            let in_archive = entry.path()?.into_owned();
            let path = in_archive.strip_prefix(PREFIX_IN_TAR)?.to_owned();
            let abs_path = output_path.join(&path);
            entry.unpack(&abs_path)?;
            Ok((in_archive, abs_path))
        })
        .filter_map(|e| e.ok())
        .for_each(|(from, to)| println!("> {} -> {}", from.display(), to.display()));
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let tar = Path::new("/tmp/archive.tar.gz");
    compress(tar)?;
    decompress(tar)
}
