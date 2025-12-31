#![no_std]

extern crate alloc;

pub use deku;
use deku::prelude::*;
use derive_more::Display;
use encore::prelude::*;

mod manifest;
pub use manifest::*;

mod writer;
pub use writer::Writer;

#[derive(Debug, Display)]
pub enum PixieError {
    #[display("{_0}")]
    Deku(DekuError),
    #[display("{_0}")]
    Encore(EncoreError),
}

impl From<DekuError> for PixieError {
    fn from(e: DekuError) -> Self {
        Self::Deku(e)
    }
}

impl From<EncoreError> for PixieError {
    fn from(e: EncoreError) -> Self {
        Self::Encore(e)
    }
}
