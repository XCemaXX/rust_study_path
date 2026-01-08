use deku::prelude::*;
use derive_more::Debug;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(magic = b"\x7FELF")]
pub struct ObjectHeader {
    #[debug(skip)]
    pub class: ElfClass,
    pub endianness: Endianness,
    pub version: u8,
    #[deku(pad_bytes_after = "8")]
    pub os_abi: OsAbi,
    pub typ: ElfType,
    pub machine: ElfMachine,
    pub version_bits: u32,
    #[debug("0x{entry_point:x}")]
    pub entry_point: u64,
    #[debug("0x{ph_offset:x}")]
    pub ph_offset: u64,
    #[debug("0x{sh_offset:x}")]
    pub sh_offset: u64,
    #[debug("0x{flags:x}")]
    pub flags: u32,
    pub hdr_size: u16,
    pub ph_entsize: u16,
    pub ph_count: u16,
    pub sh_entsize: u16,
    pub sh_count: u16,
    pub sh_nidx: u16,
}

impl ObjectHeader {
    pub const SIZE: u16 = 64;
}

#[derive(Debug, Copy, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum ElfClass {
    #[deku(id = "1")]
    Elf32,
    #[deku(id = "2")]
    Elf64,
    #[deku(id_pat = "_")]
    Other(u8),
}

#[derive(Debug, Copy, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u16")]
pub enum ElfType {
    #[deku(id = "0x2")]
    Exec,
    #[deku(id = "0x3")]
    Dyn,
    #[deku(id_pat = "_")]
    Other(u16),
}

#[derive(Debug, Copy, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum Endianness {
    #[deku(id = "0x1")]
    Little,
    #[deku(id = "0x2")]
    Big,
    #[deku(id_pat = "_")]
    Other(u8),
}

#[derive(Debug, Copy, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u16")]
pub enum ElfMachine {
    #[deku(id = "0x03")]
    X86,
    #[deku(id = "0x3e")]
    X86_64,
    #[deku(id_pat = "_")]
    Other(u16),
}

#[derive(Debug, Copy, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum OsAbi {
    #[deku(id = "0x0")]
    SysV,
    #[deku(id = "0x3")]
    NetBsd,
    #[deku(id_pat = "_")]
    Other(u8),
}
