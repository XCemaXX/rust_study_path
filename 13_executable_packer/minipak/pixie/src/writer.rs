use core::cmp::min;

use deku::DekuContainerWrite;
use encore::prelude::*;

use crate::PixieError;

const PAD_SIZE: usize = 1024;
const PAD_BUF: [u8; PAD_SIZE] = [0u8; PAD_SIZE];

type Result<T> = core::result::Result<T, PixieError>;

/// Writes to a file, maintaining a current offset
pub struct Writer {
    file: File,
    offset: u64,
}

impl Writer {
    pub fn new(path: &str, mode: u64) -> Result<Self> {
        let file = File::create(path, mode)?;
        Ok(Self { file, offset: 0 })
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.file.write_all(buf)?;
        self.offset += buf.len() as u64;
        Ok(())
    }

    /// Writes `n` bytes of padding
    pub fn pad(&mut self, mut n: u64) -> Result<()> {
        while n > 0 {
            let m = min(n, PAD_SIZE as _);
            n -= m;
            self.write_all(&PAD_BUF[..m as _])?;
        }
        Ok(())
    }

    pub fn align(&mut self, n: u64) -> Result<()> {
        let next_offset = ceil(self.offset, n);
        self.pad((next_offset - self.offset) as _)
    }

    pub fn write_deku<T: DekuContainerWrite>(&mut self, t: &T) -> Result<()> {
        self.write_all(&t.to_bytes()?)
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }
}

fn ceil(i: u64, n: u64) -> u64 {
    if i % n == 0 { i } else { (i + n) & !(n - 1) }
}
