mod addr;
mod enums;
mod parse;
mod program_header;
mod sym;

pub use crate::{addr::*, enums::*, program_header::*, sym::*};
use nom::{
    Parser as _, branch, combinator, multi,
    number::complete::{le_u16, le_u32, le_u64},
};

#[derive(Debug)]
pub struct File {
    pub r#type: Type,
    pub machine: Machine,
    pub entry_point: Addr,
    pub program_headers: Vec<ProgramHeader>,
    pub section_headers: Vec<SectionHeader>,
}

impl File {
    const MAGIC: &'static [u8] = b"\x7FELF"; // 0x7f, 0x45, 0x4c, 0x46

    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        use nom::{
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
        let (i, (ph_offset, sh_offset)) = (Addr::parse, Addr::parse).parse(i)?;
        let (i, (_flags, _hdr_size)) = (le_u32, le_u16).parse(i)?;
        let (i, (ph_entsize, ph_count)) = (u16_usize(), u16_usize()).parse(i)?;
        let (i, (sh_entsize, sh_count, _sh_nidx)) =
            (u16_usize(), u16_usize(), u16_usize()).parse(i)?;

        let ph_slices = (&full_input[ph_offset.into()..]).chunks(ph_entsize);
        let program_headers = ph_slices
            .take(ph_count)
            .map(|ph_slice| {
                let (_, ph) = ProgramHeader::parse(full_input, ph_slice)?;
                Ok(ph)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let sh_slices = (&full_input[sh_offset.into()..]).chunks(sh_entsize);
        let section_headers = sh_slices
            .take(sh_count)
            .map(|sh_slice| {
                let (_, sh) = SectionHeader::parse(sh_slice)?;
                Ok(sh)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let res = Self {
            machine,
            r#type,
            entry_point,
            program_headers,
            section_headers,
        };
        Ok((i, res))
    }

    pub fn parse_or_print_error(i: parse::Input) -> Option<Self> {
        match Self::parse(i) {
            Ok((_, file)) => Some(file),
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                eprintln!("Parsing failed:");
                for (input, err) in err.errors {
                    use nom::Offset;
                    let offset = i.offset(input);
                    eprintln!("{:?} at position {}:", err, offset);
                    eprintln!("{:>08x}: {:?}", offset, HexDump(input));
                }
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

    pub fn dynamic_table(&self) -> Option<&[DynamicEntry]> {
        match self.segment_of_type(SegmentType::Dynamic) {
            Some(ProgramHeader {
                contents: SegmentContents::Dynamic(entries),
                ..
            }) => Some(entries),
            _ => None,
        }
    }

    pub fn dynamic_entries(&self, tag: DynamicTag) -> impl Iterator<Item = Addr> + '_ {
        self.dynamic_table()
            .unwrap_or_default()
            .iter()
            .filter(move |e| e.tag == tag)
            .map(|e| e.addr)
    }

    pub fn dynamic_entry(&self, tag: DynamicTag) -> Option<Addr> {
        self.dynamic_entries(tag).next()
    }

    pub fn dynamic_entry_strings(&self, tag: DynamicTag) -> impl Iterator<Item = String> {
        self.dynamic_entries(tag)
            .filter_map(move |addr| self.get_string(addr).ok())
        // This will silently ignore strings we're not able to retrieve
    }

    pub fn read_rela_entries(&self) -> Result<Vec<Rela>, ReadRelaError> {
        use DynamicTag as DT;
        use ReadRelaError as E;

        match self.dynamic_entry(DT::Rela) {
            None => Ok(Vec::new()),
            Some(addr) => {
                let len = self.get_dynamic_entry(DT::RelaSz)?;

                let i = self.slice_at(addr).ok_or(E::RelaSegmentNotFound)?;
                let i = &i[..len.into()];
                let n = len.0 as usize / Rela::SIZE;

                match multi::many_m_n(n, n, Rela::parse).parse(i) {
                    Ok((_, rela_entires)) => Ok(rela_entires),
                    Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                        Err(E::ParsingError(format!("{err:?}")))
                    }
                    _ => {
                        unreachable!(
                            r#"we don't use any "streaming" parsers, so `nom::Err::Incomplete` seems unlikely"#
                        )
                    }
                }
            }
        }
    }

    /// Returns a slice containing the contents of the relevant Load segment
    /// starting at `mem_addr` until the end of that segment
    pub fn slice_at(&self, mem_addr: Addr) -> Option<&[u8]> {
        self.segment_at(mem_addr)
            .map(|seg| &seg.data[(mem_addr - seg.mem_range().start).into()..])
    }

    pub fn get_string(&self, offset: Addr) -> Result<String, GetSringErorr> {
        use DynamicTag as DT;
        use GetSringErorr as E;

        let addr = self.dynamic_entry(DT::StrTab).ok_or(E::StrTabNotFound)?;
        let slice = self
            .slice_at(addr + offset)
            .ok_or(E::StrTabSegmentNotFound)?;

        let string_slice = slice.split(|&c| c == 0).next().ok_or(E::StringNotFound)?;
        Ok(String::from_utf8_lossy(string_slice).into())
    }

    pub fn section_starting_at(&self, addr: Addr) -> Option<&SectionHeader> {
        self.section_headers.iter().find(|sh| sh.addr == addr)
    }

    pub fn read_syms(&self) -> Result<Vec<Sym>, ReadSymsError> {
        use DynamicTag as DT;
        use ReadSymsError as E;

        let addr = self.get_dynamic_entry(DT::SymTab)?;
        let section = self
            .section_starting_at(addr)
            .ok_or(E::SymTabSectionNotFound)?;

        let i = self.slice_at(addr).ok_or(E::SymTabSegmentNotFound)?;
        let n = (section.size.0 / section.entsize.0) as usize;

        match multi::many_m_n(n, n, Sym::parse).parse(i) {
            Ok((_, syms)) => Ok(syms),
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                Err(E::ParsingError(format!("{err:?}")))
            }
            _ => {
                unreachable!(
                    r#"we don't use any "streaming" parsers, so `nom::Err::Incomplete` seems unlikely"#
                )
            }
        }
    }

    pub fn get_dynamic_entry(&self, tag: DynamicTag) -> Result<Addr, GetDynamicEntryError> {
        self.dynamic_entry(tag)
            .ok_or(GetDynamicEntryError::NotFound(tag))
    }
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

#[derive(thiserror::Error, Debug)]
pub enum GetDynamicEntryError {
    #[error("Dynamic entry {0:?} not found")]
    NotFound(DynamicTag),
}

#[derive(Debug)]
pub struct Rela {
    pub offset: Addr,
    pub r#type: RelType,
    pub sym: u32,
    pub addend: Addr,
}

impl Rela {
    pub const SIZE: usize = 24;
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
    #[error("{0}")]
    DynamicEntryNotFound(#[from] GetDynamicEntryError),
    #[error("Rela segment not found")]
    RelaSegmentNotFound,
    #[error("Parsing error: {0}")]
    ParsingError(String),
}

#[derive(thiserror::Error, Debug)]
pub enum GetSringErorr {
    #[error("StrTab dynamic entry not found")]
    StrTabNotFound,
    #[error("StrTab segment not found")]
    StrTabSegmentNotFound,
    #[error("String not found")]
    StringNotFound,
}

#[derive(Debug)]
pub struct SectionHeader {
    pub name: Addr,
    pub r#type: u32,
    pub flags: u64,
    pub addr: Addr,
    pub off: Addr,
    pub size: Addr,
    pub link: u32,
    pub info: u32,
    pub addralign: Addr,
    pub entsize: Addr,
}

impl SectionHeader {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, (name, r#type, flags, addr, off, size, link, info, addralign, entsize)) = (
            combinator::map(le_u32, |x| Addr(x as u64)),
            le_u32,
            le_u64,
            Addr::parse,
            Addr::parse,
            Addr::parse,
            le_u32,
            le_u32,
            Addr::parse,
            Addr::parse,
        )
            .parse(i)?;
        let res = Self {
            name,
            r#type,
            flags,
            addr,
            off,
            size,
            link,
            info,
            addralign,
            entsize,
        };
        Ok((i, res))
    }
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
