use std::{
    fs::File,
    io::{BufReader, Error, Read},
    path::Path,
    sync::mpsc::channel,
};

use ring::digest::{Context, Digest, SHA256};
use thread_pool::ThreadPool;
use walkdir::WalkDir;

fn compute_digest<P: AsRef<Path>>(path: P) -> Result<(Digest, P), Error> {
    let mut buf_reader = BufReader::new(File::open(&path)?);
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = buf_reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    Ok((context.finish(), path))
}

fn main() -> Result<(), Error> {
    let (pool_send, pool) = ThreadPool::fixed_size(num_cpus::get());
    let (sender, recv) = channel();

    let source_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

    for entry in WalkDir::new(source_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().is_dir())
    {
        let path = entry.path().to_owned();
        let sender = sender.clone();
        pool_send
            .send(move || {
                let digest = compute_digest(path);
                sender.send(digest).expect("Couldn't send data");
            })
            .expect("Couldn't start task");
    }
    drop(sender);
    for msg in recv.iter() {
        println!("{:?}", msg?);
    }
    pool.shutdown();
    Ok(())
}
