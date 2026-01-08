use std::{fmt, ops::Range};

use crate::{Addr, DynamicEntry, DynamicTag, SegmentFlag, SegmentType, parse};
use enumflags2::BitFlags;
use nom::{
    Parser as _,
    combinator::{self, verify},
    multi,
};

pub struct ProgramHeader {
    pub r#type: SegmentType,
    pub flags: BitFlags<SegmentFlag>,
    pub offset: Addr,
    pub vaddr: Addr,
    pub paddr: Addr,
    pub filesz: Addr,
    pub memsz: Addr,
    pub align: Addr,
    pub contents: SegmentContents,
}

impl ProgramHeader {
    pub fn file_range(&self) -> Range<Addr> {
        self.offset..self.offset + self.filesz
    }

    pub fn mem_range(&self) -> Range<Addr> {
        self.vaddr..self.vaddr + self.memsz
    }

    pub(crate) fn parse<'a>(full_input: &'a [u8], i: parse::Input<'a>) -> parse::Result<'a, Self> {
        //fn parse<'a>(full_input: parse::Input<'_>, i: parse::Input<'a>) -> parse::Result<'a, Self> {
        let (i, (r#type, flags)) = (SegmentType::parse, SegmentFlag::parse).parse(i)?;

        let ap = Addr::parse;
        let (i, (offset, vaddr, paddr, filesz, memsz, align)) =
            (ap, ap, ap, ap, ap, ap).parse(i)?;

        let slice = &full_input[offset.into()..][..filesz.into()];
        let (_, contents) = match r#type {
            SegmentType::Dynamic => combinator::map(
                multi::many_till(
                    DynamicEntry::parse,
                    verify(DynamicEntry::parse, |e| e.tag == DynamicTag::Null),
                ),
                |(entries, _last)| SegmentContents::Dynamic(entries),
            )
            .parse(slice)?,
            _ => (slice, SegmentContents::Unknown),
        };

        let res = Self {
            r#type,
            flags,
            offset,
            vaddr,
            paddr,
            filesz,
            memsz,
            align,
            contents,
        };
        Ok((i, res))
    }
}

impl fmt::Debug for ProgramHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "file {:?} | mem {:?} | align {:?} | {} {:?}",
            self.file_range(),
            self.mem_range(),
            self.align,
            &[
                (SegmentFlag::Read, "R"),
                (SegmentFlag::Write, "W"),
                (SegmentFlag::Execute, "X")
            ]
            .iter()
            .map(|&(flag, letter)| {
                if self.flags.contains(flag) {
                    letter
                } else {
                    "."
                }
            })
            .collect::<Vec<_>>()
            .join(""),
            self.r#type,
        )
    }
}

#[derive(Debug)]
pub enum SegmentContents {
    Dynamic(Vec<DynamicEntry>),
    Unknown,
}
