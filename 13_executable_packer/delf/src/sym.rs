use std::fmt;

use crate::{Addr, GetDynamicEntryError, enums::*, parse};
use nom::{
    Parser as _, combinator,
    number::complete::{le_u8, le_u16, le_u32, le_u64},
};

#[derive(Debug, Clone)]
pub struct Sym {
    pub name: Addr,
    pub bind: SymBind,
    pub r#type: SymType,
    pub shndx: SectionIndex,
    pub value: Addr,
    pub size: u64,
}

impl Sym {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, (name, (bind, r#type), _reserved, shndx, value, size)) = (
            combinator::map(le_u32, |x| Addr(x as u64)),
            nom::bits::bits((SymBind::parse, SymType::parse)),
            le_u8,
            combinator::map(le_u16, SectionIndex),
            Addr::parse,
            le_u64,
        )
            .parse(i)?;
        let res = Self {
            name,
            bind,
            r#type,
            shndx,
            value,
            size,
        };
        Ok((i, res))
    }
}

#[derive(Clone, Copy)]
pub struct SectionIndex(pub u16);

impl SectionIndex {
    pub fn is_undef(&self) -> bool {
        self.0 == 0
    }

    pub fn is_special(&self) -> bool {
        self.0 >= 0xff00
    }

    pub fn get(&self) -> Option<usize> {
        if self.is_undef() || self.is_special() {
            None
        } else {
            Some(self.0 as usize)
        }
    }
}

impl fmt::Debug for SectionIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_special() {
            write!(f, "Special({:04x})", self.0)
        } else if self.is_undef() {
            write!(f, "Undef")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ReadSymsError {
    #[error("{0:?}")]
    DynamicEntryNotFound(#[from] GetDynamicEntryError),
    #[error("SymTab section not found")]
    SymTabSectionNotFound,
    #[error("SymTab segment not found")]
    SymTabSegmentNotFound,
    #[error("Parsing error: {0}")]
    ParsingError(String),
}
