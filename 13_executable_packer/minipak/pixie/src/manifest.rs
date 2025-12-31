use core::ops::Range;
use deku::prelude::*;

use crate::PixieError;

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(magic = b"pixoendm")]
pub struct EndMarker {
    #[deku(bytes = 8)]
    pub manifest_offset: usize,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub struct Resource {
    #[deku(bytes = 8)]
    pub offset: usize,
    #[deku(bytes = 8)]
    pub len: usize,
}

impl Resource {
    pub fn as_range(&self) -> Range<usize> {
        self.offset..self.offset + self.len
    }
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(magic = b"piximani")]
pub struct Manifest {
    // TODO: add stage2
    pub guest: Resource,
}

impl Manifest {
    pub fn read_from_full_slice(slice: &[u8]) -> Result<Self, PixieError> {
        let (_, endmarker) = EndMarker::from_bytes((&slice[slice.len() - 16..], 0)).unwrap();
        let (_, manifdest) =
            Manifest::from_bytes((&slice[endmarker.manifest_offset..], 0)).unwrap();
        Ok(manifdest)
    }
}
