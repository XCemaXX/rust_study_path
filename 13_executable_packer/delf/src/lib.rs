mod addr;
mod enums;
mod parse;
mod program_header;
mod sym;

use std::ops::Range;

pub use crate::{addr::*, enums::*, program_header::*, sym::*};
use nom::{
    Parser as _, branch, combinator, multi,
    number::complete::{le_u16, le_u32, le_u64},
};

#[derive(Debug)]
pub struct File<I>
where
    I: AsRef<[u8]>,
{
    pub input: I,
    pub contents: FileContents,
}

impl<I> std::ops::Deref for File<I>
where
    I: AsRef<[u8]>,
{
    type Target = FileContents;
    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl<I> File<I>
where
    I: AsRef<[u8]>,
{
    /// Decode a DT_RELR / Elf64_Relr table (raw `u64` words) into relocation offsets (virtual
    /// addresses).
    fn decode_relr_vaddrs(relr: &[u64]) -> Vec<Addr> {
        // Format per DT_RELR / Elf64_Relr:
        // - Even entries: an address A, relocate A, then advance by word size.
        // - Odd entries: a bitmap for the next (word_bits-1) addresses starting at `where`.
        const WORD_SIZE: u64 = 8;
        const BITS_PER_WORD: u64 = 64;
        const BITMAP_BITS: u64 = BITS_PER_WORD - 1;

        let mut out = Vec::new();
        let mut where_addr: u64 = 0;

        for &entry in relr {
            if (entry & 1) == 0 {
                where_addr = entry;
                out.push(Addr(where_addr));
                where_addr = where_addr.wrapping_add(WORD_SIZE);
            } else {
                // Bits 1..63 correspond to relocations at where_addr + i*WORD_SIZE (i=0..62)
                let bitmap = entry >> 1;
                for i in 0..BITMAP_BITS {
                    if (bitmap & (1u64 << i)) != 0 {
                        let a = where_addr.wrapping_add(i.wrapping_mul(WORD_SIZE));
                        out.push(Addr(a));
                    }
                }
                where_addr = where_addr.wrapping_add(BITMAP_BITS.wrapping_mul(WORD_SIZE));
            }
        }

        out
    }

    pub fn parse_or_print_error(input: I) -> Option<Self> {
        match FileContents::parse(input.as_ref()) {
            Ok((_, contents)) => Some(File { input, contents }),
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                eprintln!("Parsing failed:");
                for (input, err) in err.errors {
                    use nom::Offset;
                    let offset = input.offset(input);
                    eprintln!("{:?} at position {}:", err, offset);
                    eprintln!("{:>08x}: {:?}", offset, HexDump(input));
                }
                None
            }
            Err(_) => panic!("unexpected nom error"),
        }
    }

    /// Returns a slice of the input, indexed by file offsets
    pub fn file_slice(&self, addr: Addr, len: usize) -> &[u8] {
        &self.input.as_ref()[addr.into()..len]
    }

    /// Returns a slice of the input corresponding to the given section
    pub fn section_slice(&self, section: &SectionHeader) -> &[u8] {
        let fr = section.file_range();
        self.file_slice(fr.start, fr.end.into())
    }

    /// Returns a slice of the input corresponding to the given segment
    pub fn segment_slice(&self, segment: &ProgramHeader) -> &[u8] {
        let fr = segment.file_range();
        self.file_slice(fr.start, fr.end.into())
    }

    /// Returns a slice of the input, indexed by virtual addresses
    pub fn mem_slice(&self, addr: Addr, len: usize) -> Option<&[u8]> {
        self.segment_containing(addr).map(|segment| {
            let start: usize = (addr - segment.mem_range().start).into();
            &self.segment_slice(segment)[start..start + len]
        })
    }

    /// Returns an iterator of string values (or rather, u8 slices) of
    /// dynamic entries for the given tag.
    pub fn dynamic_entry_strings(&self, tag: DynamicTag) -> impl Iterator<Item = &[u8]> + '_ {
        self.dynamic_entries(tag)
            .map(move |addr| self.dynstr_entry(addr))
    }

    /// Read relocation entries from the table pointed to by `DynamicTag::Rela`
    pub fn read_rela_entries(&self) -> Result<Vec<Rela>, ReadRelaError> {
        self.read_relocations(DynamicTag::Rela, DynamicTag::RelaSz)
    }

    /// Read relocation entries from the table pointed to by `DynamicTag::JmpRel`
    pub fn read_jmp_rel_entries(&self) -> Result<Vec<Rela>, ReadRelaError> {
        self.read_relocations(DynamicTag::JmpRel, DynamicTag::PltRelSz)
    }

    /// Read a dynamic table referenced by `(addr_tag, size_tag)` and return its bytes.
    fn dynamic_table_slice(
        &self,
        addr_tag: DynamicTag,
        size_tag: DynamicTag,
        seg_err: fn() -> ReadRelaError,
    ) -> Result<Option<&[u8]>, ReadRelaError> {
        let Some(addr) = self.dynamic_entry(addr_tag) else {
            return Ok(None);
        };
        let len = self.get_dynamic_entry(size_tag)?;
        if len.0 == 0 {
            return Ok(None);
        }
        let bytes = self.mem_slice(addr, len.into()).ok_or_else(seg_err)?;
        Ok(Some(bytes))
    }

    /// Read compressed relative relocations (DT_RELR / Elf64_Relr).
    ///
    /// Returns the raw `u64` entries of the RELR table. The caller is responsible for
    /// interpreting the bitmap encoding.
    pub fn read_relr_entries(&self) -> Result<Vec<u64>, ReadRelaError> {
        use ReadRelaError as E;

        let Some(bytes) =
            self.dynamic_table_slice(DynamicTag::Relr, DynamicTag::RelrSz, || E::RelrSegmentNotFound)?
        else {
            return Ok(Vec::new());
        };

        if bytes.len() % 8 != 0 {
            return Err(E::ParsingError(format!(
                "DT_RELR size is not a multiple of 8 (got {})",
                bytes.len()
            )));
        }

        Ok(bytes
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().expect("chunk size is 8")))
            .collect())
    }

    /// Read compressed relative relocations (DT_RELR) and decode them into the list of
    /// relocation offsets (virtual addresses).
    ///
    /// Note: RELR's addend is stored at the relocation location (REL-style). Converting to
    /// full `Rela` entries requires a loader/runtime view of memory, so this API only returns
    /// offsets.
    pub fn read_relr_vaddrs(&self) -> Result<Vec<Addr>, ReadRelaError> {
        let raw = self.read_relr_entries()?;
        Ok(Self::decode_relr_vaddrs(&raw))
    }

    /// Read symbols from the given section (internal)
    fn read_symbol_table(&self, section_type: SectionType) -> Result<Vec<Sym>, ReadSymsError> {
        let Some(section) = self.section_of_type(section_type) else {
            return Ok(Vec::new());
        };

        let i = self.section_slice(section);
        let n = i.len() / section.entsize.0 as usize;
        match multi::many_m_n(n, n, Sym::parse).parse(i) {
            Ok((_, syms)) => Ok(syms),
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                Err(ReadSymsError::ParsingError(format!("{err:?}")))
            }
            _ => unreachable!(),
        }
    }

    /// Read symbols from the ".dynsym" section (loader view)
    pub fn read_dynsym_entries(&self) -> Result<Vec<Sym>, ReadSymsError> {
        self.read_symbol_table(SectionType::DynSym)
    }

    /// Read symbols from the ".symtab" section (linker view)
    pub fn read_symtab_entries(&self) -> Result<Vec<Sym>, ReadSymsError> {
        self.read_symbol_table(SectionType::SymTab)
    }

    // Returns a null-terminated "string" from the ".shstrtab" section as an u8 slice
    pub fn shstrtab_entry(&self, offset: Addr) -> &[u8] {
        let section = &self.contents.section_headers[self.contents.shstrndx];
        let slice = &self.section_slice(section)[offset.into()..];
        slice.split(|&c| c == 0).next().unwrap_or_default()
    }

    /// Get a section by name
    pub fn section_by_name(&self, name: &[u8]) -> Option<&SectionHeader> {
        self.section_headers
            .iter()
            .find(|sh| self.shstrtab_entry(sh.name) == name)
    }

    /// Returns an entry from a string table contained in the section with a given name
    fn string_table_entry(&self, name: &[u8], offset: Addr) -> &[u8] {
        self.section_by_name(name)
            .map(|section| {
                let slice = &self.section_slice(section)[offset.into()..];
                slice.split(|&c| c == 0).next().unwrap_or_default()
            })
            .unwrap_or_default()
    }

    /// Returns a null-terminated "string" from the ".strtab" section as an u8 slice
    pub fn strtab_entry(&self, offset: Addr) -> &[u8] {
        self.string_table_entry(b".strtab", offset)
    }

    /// Returns a null-terminated "string" from the ".dynstr" section as an u8 slice
    pub fn dynstr_entry(&self, offset: Addr) -> &[u8] {
        self.string_table_entry(b".dynstr", offset)
    }

    fn read_relocations(
        &self,
        addr_tag: DynamicTag,
        size_tag: DynamicTag,
    ) -> Result<Vec<Rela>, ReadRelaError> {
        use ReadRelaError as E;

        let Some(i) = self.dynamic_table_slice(addr_tag, size_tag, || E::RelaSegmentNotFound)? else {
            return Ok(Vec::new());
        };
        let n = i.len() / Rela::SIZE;

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

#[derive(Debug)]
pub struct FileContents {
    pub r#type: Type,
    pub machine: Machine,
    pub entry_point: Addr,
    pub program_headers: Vec<ProgramHeader>,
    pub section_headers: Vec<SectionHeader>,
    pub shstrndx: usize,
}

impl FileContents {
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
        let (i, (sh_entsize, sh_count, sh_nidx)) =
            (u16_usize(), u16_usize(), u16_usize()).parse(i)?;

        let ph_slices = (full_input[ph_offset.into()..]).chunks(ph_entsize);
        let program_headers = ph_slices
            .take(ph_count)
            .map(|ph_slice| {
                let (_, ph) = ProgramHeader::parse(full_input, ph_slice)?;
                Ok(ph)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let sh_slices = (full_input[sh_offset.into()..]).chunks(sh_entsize);
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
            shstrndx: sh_nidx as _,
        };
        Ok((i, res))
    }

    /// Returns the first segment of a given type
    pub fn segment_of_type(&self, r#type: SegmentType) -> Option<&ProgramHeader> {
        self.program_headers.iter().find(|ph| ph.r#type == r#type)
    }

    /// Returns the first section of a given type
    pub fn section_of_type(&self, r#type: SectionType) -> Option<&SectionHeader> {
        self.section_headers.iter().find(|sh| sh.r#type == r#type)
    }

    /// Attempts to find a Load segment whose memory range contains the given virtual address
    pub fn segment_containing(&self, addr: Addr) -> Option<&ProgramHeader> {
        self.program_headers
            .iter()
            .find(|ph| ph.r#type == SegmentType::Load && ph.mem_range().contains(&addr))
    }

    /// Attempts to find the Dynamic segment and return its entries as a slice
    pub fn dynamic_table(&self) -> Option<&[DynamicEntry]> {
        match self.segment_of_type(SegmentType::Dynamic) {
            Some(ProgramHeader {
                contents: SegmentContents::Dynamic(entries),
                ..
            }) => Some(entries),
            _ => None,
        }
    }

    /// Returns an iterator of all dynamic entries with the given tag.
    /// Especially useful with DynamicTag::Needed
    pub fn dynamic_entries(&self, tag: DynamicTag) -> impl Iterator<Item = Addr> + '_ {
        self.dynamic_table()
            .unwrap_or_default()
            .iter()
            .filter(move |e| e.tag == tag)
            .map(|e| e.addr)
    }

    /// Returns the value of the first dynamic entry with the given tag, or None
    pub fn dynamic_entry(&self, tag: DynamicTag) -> Option<Addr> {
        self.dynamic_entries(tag).next()
    }

    /// Returns the value of the first dynamic entry with the given tag, or an error
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
    #[error("Relr segment not found")]
    RelrSegmentNotFound,
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
    pub r#type: SectionType,
    pub flags: u64,
    pub addr: Addr,
    pub offset: Addr,
    pub size: Addr,
    pub link: u32,
    pub info: u32,
    pub addralign: Addr,
    pub entsize: Addr,
}

impl SectionHeader {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, (name, r#type, flags, addr, offset, size, link, info, addralign, entsize)) = (
            combinator::map(le_u32, |x| Addr(x as u64)),
            SectionType::parse,
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
            offset,
            size,
            link,
            info,
            addralign,
            entsize,
        };
        Ok((i, res))
    }

    pub fn file_range(&self) -> Range<Addr> {
        self.offset..self.offset + self.size
    }

    pub fn mem_range(&self) -> Range<Addr> {
        self.addr..self.addr + self.size
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
