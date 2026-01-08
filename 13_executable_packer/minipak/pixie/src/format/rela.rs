use deku::prelude::*;
use derive_more::Debug;

#[derive(Debug, DekuRead, DekuWrite, Clone)]
pub struct Rela {
    pub offset: u64,
    pub typ: RelType,
    pub sym: u32,
    pub addend: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u32")]
pub enum RelType {
    #[deku(id = "0")]
    Null,
    #[deku(id = "1")]
    _64,
    #[deku(id = "6")]
    GlobDat,
    #[deku(id = "7")]
    JumpSlot,
    #[deku(id = "8")]
    Relative,
    #[deku(id = "16")]
    DtpMod64,
    #[deku(id_pat = "_")]
    Other(u32),
}
