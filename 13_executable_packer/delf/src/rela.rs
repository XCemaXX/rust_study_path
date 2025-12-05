use crate::enums::*;
use crate::parse::ErrorKind;
use crate::{Addr, parse};
use nom::{Parser as _, branch, combinator, number::complete::le_u32};

#[derive(Debug)]
pub struct Rela {
    pub offset: Addr,
    pub r#type: RelType,
    pub sym: u32,
    pub addend: Addr,
}

impl Rela {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        combinator::map(
            (Addr::parse, RelType::parse, le_u32, Addr::parse),
            |(offset, r#type, sym, addend)| Rela {
                offset,
                r#type,
                sym,
                addend,
            },
        )
        .parse(i)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelType {
    Known(KnownRelType),
    Unknown(u32),
}

impl RelType {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        branch::alt((
            combinator::map(KnownRelType::parse, Self::Known),
            combinator::map(le_u32, Self::Unknown),
        ))
        .parse(i)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ReadRelaError {
    #[error("Rela dynamic entry not found")]
    RelaNotFound,
    #[error("RelaSz dynamic entry not found")]
    RelaSzNotFound,
    #[error("RelaSegmentNotFound dynamic entry not found")]
    RelaSegmentNotFound,
    #[error("RelaEnt dynamic entry not found")]
    RelaEntNotFound,
    #[error("Parsing error")]
    ParsingError(ErrorKind),
}
