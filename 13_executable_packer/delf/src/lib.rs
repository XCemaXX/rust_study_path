mod enums;
mod parse;

use std::{fmt, ops::Range};

pub use crate::enums::*;
use derive_more::*;
use enumflags2::BitFlags;
use nom::{
    Offset as _, Parser as _,
    combinator::{self, verify},
    error, multi,
    number::complete::{le_u16, le_u32, le_u64},
};

#[derive(Debug)]
pub struct File {
    pub r#type: Type,
    pub machine: Machine,
    pub entry_point: Addr,
    pub program_headers: Vec<ProgramHeader>,
}

impl File {
    const MAGIC: &'static [u8] = b"\x7FELF"; // 0x7f, 0x45, 0x4c, 0x46

    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        use nom::{
            branch,
            bytes::complete::{tag, take},
            error::context,
        };
        let full_input = i;

        let (i, _) = (
            context("Magic", tag(Self::MAGIC)),
            context("Class", tag([0x2].as_slice())),
            context("Endianness", tag([0x1].as_slice())),
            context("Version", tag([0x1].as_slice())),
            context(
                "OS ABI",
                branch::alt((tag([0x0].as_slice()), tag([0x3].as_slice()))),
            ),
            context("ABI Version", take(1usize)),
            context("Padding", take(7_usize)),
        )
            .parse(i)?;

        let (i, (r#type, machine)) = (Type::parse, Machine::parse).parse(i)?;
        let (i, _) = context("Version (bis)", combinator::verify(le_u32, |&x| x == 1)).parse(i)?;
        let (i, entry_point) = Addr::parse(i)?;

        let u16_usize = || combinator::map(le_u16, |x| x as usize);

        // ph = program header, sh = section header
        let (i, (ph_offset, _sh_offset)) = (Addr::parse, Addr::parse).parse(i)?;
        let (i, (_flags, _hdr_size)) = (le_u32, le_u16).parse(i)?;
        let (i, (ph_entsize, ph_count)) = (u16_usize(), u16_usize()).parse(i)?;
        let (i, (_sh_entsize, _sh_count, _sh_nidx)) =
            (u16_usize(), u16_usize(), u16_usize()).parse(i)?;

        let ph_slices = (&full_input[ph_offset.into()..]).chunks(ph_entsize);
        let program_headers = ph_slices
            .take(ph_count)
            .map(|ph_slice| {
                let (_, ph) = ProgramHeader::parse(full_input, ph_slice)?;
                Ok(ph)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let res = Self {
            machine,
            r#type,
            entry_point,
            program_headers,
        };
        Ok((i, res))
    }

    pub fn parse_or_print_error(i: parse::Input) -> Option<Self> {
        match Self::parse(i) {
            Ok((_, file)) => Some(file),
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                let offset = i.offset(err.input);
                eprintln!("Parsing failed at {:?} at position {offset}", err.code);
                eprintln!("{:?}", HexDump(err.input));
                None
            }
            Err(_) => panic!("unexpected nom error"),
        }
    }

    pub fn segment_at(&self, addr: Addr) -> Option<&ProgramHeader> {
        self.program_headers
            .iter()
            .filter(|ph| ph.r#type == SegmentType::Load)
            .find(|ph| ph.mem_range().contains(&addr))
    }

    pub fn segment_of_type(&self, r#type: SegmentType) -> Option<&ProgramHeader> {
        self.program_headers.iter().find(|ph| ph.r#type == r#type)
    }

    pub fn dynamic_entry(&self, tag: DynamicTag) -> Option<Addr> {
        match self.segment_of_type(SegmentType::Dynamic) {
            Some(ProgramHeader {
                contents: SegmentContents::Dynamic(entries),
                ..
            }) => entries.iter().find(|e| e.tag == tag).map(|e| e.addr),
            _ => None,
        }
    }

    pub fn read_rela_entries(&self) -> Result<Vec<Rela>, ReadRelaError> {
        use DynamicTag as DT;
        use ReadRelaError as E;

        let addr = self.dynamic_entry(DT::Rela).ok_or(E::RelaNotFound)?;
        let len = self.dynamic_entry(DT::RelaSz).ok_or(E::RelaSzNotFound)?;
        let seg = self.segment_at(addr).ok_or(E::RelaSegmentNotFound)?;

        let i = &seg.data[(addr - seg.mem_range().start).into()..][..len.into()];
        match multi::many0(Rela::parse).parse(i) {
            Ok((_, rela_entires)) => Ok(rela_entires),
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                let error_kind = err.code;
                Err(E::ParsingError(error_kind))
            }
            _ => {
                unreachable!(
                    r#"we don't use any "streaming" parsers, so `nom::Err::Incomplete` seems unlikely"#
                )
            }
        }
    }
}

pub struct ProgramHeader {
    pub r#type: SegmentType,
    pub flags: BitFlags<SegmentFlag>,
    pub offset: Addr,
    pub vaddr: Addr,
    pub paddr: Addr,
    pub filesz: Addr,
    pub memsz: Addr,
    pub align: Addr,
    pub data: Vec<u8>,
    pub contents: SegmentContents,
}

impl ProgramHeader {
    pub fn file_range(&self) -> Range<Addr> {
        self.offset..self.offset + self.filesz
    }

    pub fn mem_range(&self) -> Range<Addr> {
        self.vaddr..self.vaddr + self.memsz
    }

    fn parse<'a>(full_input: &'a [u8], i: parse::Input<'a>) -> parse::Result<'a, Self> {
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
            data: full_input[offset.into()..][..filesz.into()].to_vec(),
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

#[derive(Debug)]
pub struct DynamicEntry {
    pub tag: DynamicTag,
    pub addr: Addr,
}

impl DynamicEntry {
    fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, (tag, addr)) = (DynamicTag::parse, Addr::parse).parse(i)?;
        Ok((i, Self { tag, addr }))
    }
}

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

#[derive(thiserror::Error, Debug)]
pub enum ReadRelaError {
    #[error("Rela dynamic entry not found")]
    RelaNotFound,
    #[error("RelaSz dynamic entry not found")]
    RelaSzNotFound,
    #[error("RelaSegmentNotFound dynamic entry not found")]
    RelaSegmentNotFound,
    #[error("Parsing error")]
    ParsingError(error::ErrorKind), // hm
}

pub struct HexDump<'a>(&'a [u8]);

impl<'a> std::fmt::Debug for HexDump<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &x in self.0.iter().take(20) {
            write!(f, "{x:02x} ")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, Sub)]
pub struct Addr(pub u64);

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08x}", self.0)
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<u64> for Addr {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Into<usize> for Addr {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Addr {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        combinator::map(le_u64, From::from).parse(i)
    }
}

#[cfg(test)]
mod tests {
    use enumflags2::BitFlags;

    use super::*;

    #[test]
    fn type_to_u16() {
        assert_eq!(Type::Exec as u16, 0x2);
    }

    #[test]
    fn type_from_u16() {
        assert_eq!(super::Type::try_from(0x3), Ok(Type::Dyn));
        assert!(super::Type::try_from(0xf00d).is_err());
    }

    #[test]
    fn try_enums() {
        assert_eq!(Machine::X86_64 as u16, 0x3E);
        assert_eq!(Machine::try_from(0x3E), Ok(Machine::X86_64));
        assert!(Machine::try_from(0xFA).is_err());
    }

    #[test]
    fn try_bitflag() {
        let flags_integer = 6_u32;
        let flags = BitFlags::<SegmentFlag>::from_bits(flags_integer).unwrap();
        assert_eq!(flags, SegmentFlag::Read | SegmentFlag::Write);
        assert_eq!(flags.bits(), flags_integer);
        assert!(BitFlags::<SegmentFlag>::from_bits(1992).is_err());
    }
}
