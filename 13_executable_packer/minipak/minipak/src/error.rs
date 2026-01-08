use encore::prelude::*;
use pixie::{PixieError, deku::DekuError};

use derive_more::{Debug, Display};

#[derive(Display, Debug)]
pub enum Error {
    #[display("{_0}")]
    Encore(EncoreError),
    #[display("deku error: `{_0}`")]
    Deku(DekuError),
    #[display("pixie error: `{_0}`")]
    Pixie(PixieError),
}

impl From<EncoreError> for Error {
    fn from(e: EncoreError) -> Self {
        Self::Encore(e)
    }
}

impl From<DekuError> for Error {
    fn from(e: DekuError) -> Self {
        Self::Deku(e)
    }
}

impl From<PixieError> for Error {
    fn from(e: PixieError) -> Self {
        Self::Pixie(e)
    }
}
