use core::{mem::MaybeUninit, ops::Deref};

use alloc::{format, string::String};

use crate::{
    error::EncoreEror,
    memmap::{FileOpts, MmapOptions},
    prelude::{MmapProt, OpenFlags},
    syscall::{self, FileDescriptor, Stat},
};

pub struct File {
    path: String,
    fd: FileDescriptor,
}

#[allow(clippy::len_without_is_empty)]
impl File {
    // Opens a file (read-only)
    pub fn open(path: &str) -> Result<Self, EncoreEror> {
        Self::raw_open(path, OpenFlags::RDONLY, 0)
    }

    pub fn create(path: &str, mode: u64) -> Result<Self, EncoreEror> {
        Self::raw_open(
            path,
            OpenFlags::RDWR | OpenFlags::CREAT | OpenFlags::TRUNC,
            mode,
        )
    }

    fn raw_open(path: &str, flags: OpenFlags, mode: u64) -> Result<Self, EncoreEror> {
        let nul_path = format!("{}\0", path);
        let fd = unsafe { syscall::open(nul_path.as_ptr(), flags, mode) };
        if (fd.0 as i64) < 0 {
            return Err(EncoreEror::Open(path.into()));
        }

        Ok(Self {
            path: path.into(),
            fd,
        })
    }

    pub fn write_all(&mut self, mut buf: &[u8]) -> Result<(), EncoreEror> {
        while !buf.is_empty() {
            let written = unsafe { syscall::write(self.fd, buf.as_ptr(), buf.len() as u64) };
            if written as i64 == -1 {
                return Err(EncoreEror::Write(self.path.clone()));
            }
            buf = &buf[written as usize..];
        }
        Ok(())
    }

    pub fn len(&self) -> Result<u64, EncoreEror> {
        let mut stat = MaybeUninit::<Stat>::uninit();
        let ret = unsafe { syscall::fstat(self.fd, stat.as_mut_ptr()) };
        if ret != 0 {
            return Err(EncoreEror::Stat(self.path.clone()));
        }
        let stat = unsafe { stat.assume_init() };
        Ok(stat.size)
    }

    pub fn fd(&self) -> FileDescriptor {
        self.fd
    }

    pub fn map(&self) -> Result<Map<'_>, EncoreEror> {
        let len = self.len()?;
        let self_data = MmapOptions::new(len)
            .file(FileOpts {
                fd: self.fd,
                offset: 0,
            })
            .prot(MmapProt::READ)
            .map()? as *const u8;
        let data = unsafe { core::slice::from_raw_parts(self_data, len as _) };
        Ok(Map { _file: self, data })
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe { syscall::close(self.fd) };
    }
}

pub struct Map<'a> {
    _file: &'a File,
    data: &'a [u8],
}

impl Drop for Map<'_> {
    fn drop(&mut self) {
        unsafe { syscall::munmap(self.data.as_ptr(), self.data.len() as _) };
    }
}

impl AsRef<[u8]> for Map<'_> {
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl Deref for Map<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}
