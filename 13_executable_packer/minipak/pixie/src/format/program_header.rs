use core::ops::Range;

use deku::prelude::*;

use derive_more::Debug;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct ProgramHeader {
    pub typ: SegmentType,
    #[debug("0x{flags:x}")]
    pub flags: u32,
    #[debug("0x{offset:x}")]
    pub offset: u64,
    #[debug("0x{vaddr:x}")]
    pub vaddr: u64,
    #[debug("0x{paddr:x}")]
    pub paddr: u64,
    #[debug("0x{filesz:x}")]
    pub filesz: u64,
    #[debug("0x{memsz:x}")]
    pub memsz: u64,
    #[debug("0x{align:x}")]
    pub align: u64,
}

impl ProgramHeader {
    pub const SIZE: u16 = 56;
    pub const EXECUTE: u32 = 1;
    pub const WRITE: u32 = 2;
    pub const READ: u32 = 4;

    /// Returns a range that spans from offset to offset+filesz
    pub fn file_range(&self) -> Range<usize> {
        let start = self.offset as usize;
        let len = self.filesz as usize;
        let end = start + len;
        start..end
    }

    /// Returns a range that spans from vaddr to vaddr+memsz
    pub fn mem_range(&self) -> Range<u64> {
        let start = self.vaddr;
        let len = self.memsz;
        let end = start + len;
        start..end
    }
}

#[derive(Debug, Copy, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u32")]
pub enum SegmentType {
    #[deku(id = "0x0")]
    Null,
    #[deku(id = "0x1")]
    Load,
    #[deku(id = "0x2")]
    Dynamic,
    #[deku(id = "0x3")]
    Interp,
    #[deku(id = "0x4")]
    Note,
    #[deku(id = "0x5")]
    ShLib,
    #[deku(id = "0x6")]
    PHdr,
    #[deku(id = "0x7")]
    TLS,
    #[deku(id = "0x6000_0000")]
    LoOS,
    #[deku(id = "0x6FFF_FFFF")]
    HiOS,
    #[deku(id = "0x7000_0000")]
    LoProc,
    #[deku(id = "0x7FFF_FFFF")]
    HiProc,
    #[deku(id = "0x6474_E550")]
    GnuEhFrame,
    #[deku(id = "0x6474_E551")]
    GnuStack,
    #[deku(id = "0x6474_E552")]
    GnuRelRo,
    #[deku(id = "0x6474_E553")]
    GnuProperty,
    #[deku(id_pat = "_")]
    Other(u32),
}
