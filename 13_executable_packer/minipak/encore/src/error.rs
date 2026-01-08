use derive_more::Display;

use alloc::string::String;

#[derive(Debug, Display)]
pub enum EncoreError {
    #[display("Could not open file {_0}")]
    Open(String),
    #[display("Could not write to file {_0}")]
    Write(String),
    #[display("Could not statfile {_0}")]
    Stat(String),
    #[display("mmap fixed address provided was not aligned to 0x1000: {_0}")]
    MmapMemUnaligned(u64),
    #[display("mmap file offset provided was not aligned to 0x1000: {_0}")]
    MmapFileUnaligned(u64),
    #[display("mmap syscall failed")]
    MmapFailed,
}
