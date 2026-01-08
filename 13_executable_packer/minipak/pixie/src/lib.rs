#![no_std]

extern crate alloc;

use core::ops::Range;

use alloc::boxed::Box;
pub use deku;
use deku::prelude::*;
use derive_more::Display;
use encore::prelude::*;

mod manifest;
pub use manifest::*;

mod writer;
pub use writer::Writer;

mod format;
pub use format::*;

mod launch;
pub use launch::*;

#[derive(Debug, Display)]
pub enum PixieError {
    #[display("{_0}")]
    Deku(DekuError),
    #[display("{_0}")]
    Encore(EncoreError),
    #[display("no segments found")]
    NoSegmentsFound,
    #[display("could not find segment of type `{_0:?}`")]
    SegmentNotFound(SegmentType),
    #[display("cannot map non-relocatable object at fixed position")]
    CannotMapNonRelocatableObjectAtFixedPosition,
    #[display("cannot relocate non-relocatable object")]
    CannotRelocateNonRelocatableObject,
    #[display("could not find dynamic entry of type `{_0:?}`")]
    DynamicEntryNotFound(DynamicTagType),
    #[display("unsupported relocation type`{_0:?}`")]
    UnsupportedRela(Rela),
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

pub struct Object<'a> {
    header: ObjectHeader,
    slice: &'a [u8],
    segments: Segments<'a>,
}

impl<'a> Object<'a> {
    /// Read an ELF object from a given slice
    pub fn new(slice: &'a [u8]) -> Result<Self, PixieError> {
        let input = (slice, 0);
        let (_, header) = ObjectHeader::from_bytes(input)?;

        let segments = {
            let mut segments = Segments::default();
            let mut input = (&slice[header.ph_offset as usize..], 0);
            for _ in 0..header.ph_count {
                let (rest, ph) = ProgramHeader::from_bytes(input)?;
                segments.items.push(Segment::new(ph, slice));
                input = rest;
            }
            segments
        };

        Ok(Self {
            header,
            slice,
            segments,
        })
    }

    pub fn header(&self) -> &ObjectHeader {
        &self.header
    }

    pub fn slice(&self) -> &[u8] {
        self.slice
    }

    pub fn segments(&self) -> &Segments<'_> {
        &self.segments
    }

    pub fn read_dynamic_entries(&self) -> Result<DynamicEntries<'a>, PixieError> {
        let dyn_seg = self.segments.find(SegmentType::Dynamic)?;
        let mut entries = DynamicEntries::default();
        let mut input = (dyn_seg.slice(), 0);
        loop {
            let (rest, tag) = DynamicTag::from_bytes(input)?;
            if tag.typ == DynamicTagType::Null {
                break;
            }
            entries.items.push(DynamicEntry {
                tag,
                full_slice: &self.slice,
            });
            input = rest;
        }
        Ok(entries)
    }
}

pub struct Segment<'a> {
    header: ProgramHeader,
    /// The slice for this segment
    slice: &'a [u8],
}

impl<'a> Segment<'a> {
    pub fn new(header: ProgramHeader, full_slice: &'a [u8]) -> Self {
        let start = header.offset as usize;
        let len = header.filesz as usize;
        Self {
            header,
            slice: &full_slice[start..][..len],
        }
    }

    pub fn header(&self) -> &ProgramHeader {
        &self.header
    }

    pub fn slice(&self) -> &[u8] {
        self.slice
    }

    pub fn typ(&self) -> SegmentType {
        self.header.typ
    }
}

#[derive(Default)]
pub struct Segments<'a> {
    items: Vec<Segment<'a>>,
}

impl<'a> Segments<'a> {
    /// Returns all segments
    pub fn all(&self) -> &[Segment<'_>] {
        &self.items
    }

    /// Returns all segments of a certain type
    pub fn of_type(&self, typ: SegmentType) -> impl Iterator<Item = &Segment<'a>> + '_ {
        self.items.iter().filter(move |s| s.typ() == typ)
    }

    /// Returns the first segment of a given type or none if none matched
    pub fn find(&self, typ: SegmentType) -> Result<&Segment<'_>, PixieError> {
        self.of_type(typ)
            .next()
            .ok_or(PixieError::SegmentNotFound(typ))
    }

    /// Returns a 4K-aligned convex hull of all the load segments
    pub fn load_convex_hull(&self) -> Result<Range<u64>, PixieError> {
        self.of_type(SegmentType::Load)
            .map(|s| s.header().mem_range())
            .reduce(|a, b| a.start.min(b.start)..a.end.max(b.end))
            .ok_or(PixieError::NoSegmentsFound)
    }
}

/// An ELF object mapped into memory
pub struct MappedObject<'a> {
    object: &'a Object<'a>,
    /// Load convex hull
    hull: Range<u64>,
    /// Difference between the start of the load convex hull
    /// and where it's actually mapped. For relocatable objects,
    /// it's the base we picked. For non-relocatable objects,
    /// it's zero.
    base_offset: u64,
    /// Memory allocated for the object in question
    mem: &'a mut [u8],
}

impl<'a> MappedObject<'a> {
    pub fn new(object: &'a Object, mut at: Option<u64>) -> Result<Self, PixieError> {
        let hull = object.segments().load_convex_hull()?;
        let is_relocatable = hull.start == 0;

        if !is_relocatable {
            if at.is_some() {
                return Err(PixieError::CannotMapNonRelocatableObjectAtFixedPosition);
            }
            at = Some(hull.start);
        }
        let mem_len = hull.end - hull.start;

        let mut map_opts = MmapOptions::new(mem_len);
        map_opts.prot(MmapProt::READ | MmapProt::WRITE | MmapProt::EXEC);
        if let Some(at) = at {
            map_opts.at(at);
        }

        let res = map_opts.map()?;
        let base_offset = if is_relocatable { res } else { 0 };
        let mem = unsafe { core::slice::from_raw_parts_mut(res as _, mem_len as _) };
        let mut mapped = Self {
            hull,
            object,
            base_offset,
            mem,
        };
        mapped.copy_load_segments();
        Ok(mapped)
    }

    fn copy_load_segments(&mut self) {
        for s in self.object.segments().of_type(SegmentType::Load) {
            let mem_start = self.vaddr_to_mem_offset(s.header().vaddr);
            let dst = &mut self.mem[mem_start..][..s.slice().len()];
            dst.copy_from_slice(s.slice());
        }
    }

    /// Convert a vaddr to a memory offset
    pub fn vaddr_to_mem_offset(&self, vaddr: u64) -> usize {
        (vaddr - self.hull.start) as _
    }

    /// Returns a view of (potentially relocated) `mem` for a given range
    pub fn vaddr_slice(&self, range: Range<u64>) -> &[u8] {
        &self.mem[self.vaddr_to_mem_offset(range.start)..self.vaddr_to_mem_offset(range.end)]
    }

    /// Returns true if the object's base offset is zero, which we assume
    /// means it can be mapped anywhere.
    pub fn is_relocatable(&self) -> bool {
        self.hull.start == 0
        // self.base_offset == 0 // in guide. But It should be changed
    }

    /// Returns the offset between the object's base and where we loaded it
    pub fn base_offset(&self) -> u64 {
        self.base_offset
    }

    /// Returns the base address for this executable
    pub fn base(&self) -> u64 {
        self.mem.as_ptr() as _
    }

    /// Apply relocations with the given base offset
    pub fn relocate(&mut self, base_offset: u64) -> Result<(), PixieError> {
        if !self.is_relocatable() {
            return Err(PixieError::CannotRelocateNonRelocatableObject);
        }

        let dyn_entries = self.object.read_dynamic_entries()?;
        let syms = dyn_entries.syms()?;

        let relas = dyn_entries
            .find(DynamicTagType::Rela)?
            .parse_all(dyn_entries.find(DynamicTagType::RelaSz)?);
        let plt_relas: Box<dyn Iterator<Item = _>> = match dyn_entries.find(DynamicTagType::JmpRel)
        {
            Ok(jmprel) => Box::new(jmprel.parse_all(dyn_entries.find(DynamicTagType::PltRelSz)?)),
            Err(_) => Box::new(core::iter::empty()) as _,
        };

        for rela in relas.chain(plt_relas) {
            let rela = rela?;
            self.apply_rela(&syms, &rela, base_offset)?;
        }

        Ok(())
    }

    fn apply_rela(&mut self, syms: &Syms, rela: &Rela, base_offset: u64) -> Result<(), PixieError> {
        match rela.typ {
            RelType::_64 | RelType::GlobDat | RelType::JumpSlot | RelType::Relative => {}
            _ => {
                return Err(PixieError::UnsupportedRela(rela.clone()));
            }
        }
        let (sym, _) = syms.nth(rela.sym as _)?;
        let value = base_offset + sym.value + rela.addend;

        let mem_offset = self.vaddr_to_mem_offset(rela.offset);
        unsafe {
            let target = self.mem.as_ptr().add(mem_offset) as *mut u64;
            *target = value;
        }
        Ok(())
    }

    /// Returns the (non-relocated) vaddr of a symbol by name
    pub fn lookup_sym(&self, name: &str) -> Result<Sym, PixieError> {
        let dyn_entries = self.object.read_dynamic_entries()?;
        dyn_entries.syms()?.by_name(name)
    }
}

#[derive(Default)]
pub struct DynamicEntries<'a> {
    items: Vec<DynamicEntry<'a>>,
}

impl<'a> DynamicEntries<'a> {
    // Returns a slice of all entries
    pub fn all(&self) -> &[DynamicEntry<'_>] {
        &self.items
    }

    /// Returns all entries of a certain type
    pub fn of_type(&self, typ: DynamicTagType) -> impl Iterator<Item = &DynamicEntry<'a>> + '_ {
        self.items.iter().filter(move |e| e.typ() == typ)
    }

    /// Returns the first entry of a given type or none if none matched
    pub fn find(&self, typ: DynamicTagType) -> Result<&DynamicEntry<'_>, PixieError> {
        self.of_type(typ)
            .next()
            .ok_or(PixieError::DynamicEntryNotFound(typ))
    }

    /// Constructs an instance of `Syms`. Requires the presence of the `SymTab`,
    /// `SymEnt` and `StrTab` dynamic entries.
    pub fn syms(&self) -> Result<Syms<'_>, PixieError> {
        Ok(Syms {
            symtab: self.find(DynamicTagType::SymTab)?,
            syment: self.find(DynamicTagType::SymEnt)?,
            strtab: self.find(DynamicTagType::StrTab)?,
        })
    }
}

pub struct DynamicEntry<'a> {
    tag: DynamicTag,
    full_slice: &'a [u8],
}

impl<'a> DynamicEntry<'a> {
    pub fn typ(&self) -> DynamicTagType {
        self.tag.typ
    }

    /// Returns a slice of the full file starting with this entry interpreted as
    /// an offset.
    pub fn as_slice(&self) -> &[u8] {
        &self.full_slice[self.as_usize()..]
    }

    /// Returns this entry's value
    pub fn as_usize(&self) -> usize {
        self.as_u64() as _
    }

    /// Returns this entry's value
    pub fn as_u64(&self) -> u64 {
        self.tag.addr
    }

    pub fn parse_all<T>(
        &'a self,
        len: &DynamicEntry<'a>,
    ) -> impl Iterator<Item = Result<T, PixieError>> + 'a
    where
        T: DekuContainerRead<'a>,
    {
        let slice = &self.as_slice()[..len.as_usize()];
        let mut input = (slice, 0);

        core::iter::from_fn(move || -> Option<Result<T, PixieError>> {
            if input.0.is_empty() {
                return None;
            }

            let (rest, t) = match T::from_bytes(input) {
                Ok(x) => x,
                Err(e) => return Some(Err(e.into())),
            };
            input = rest;
            Some(Ok(t))
        })
    }

    /// Parses the nth `T` record, using `self` as the start of the input, and
    /// `record_len` as the record length.
    pub fn parse_nth<T>(&'a self, record_len: &DynamicEntry<'a>, n: usize) -> Result<T, DekuError>
    where
        T: DekuContainerRead<'a>,
    {
        let slice = &self.as_slice()[(record_len.as_usize() * n)..];
        let input = (slice, 0);
        let (_, t) = T::from_bytes(input)?;
        Ok(t)
    }
}

/// Allows reading symbols out of an ELF file
pub struct Syms<'a> {
    /// Indicates the start of the symbol table
    symtab: &'a DynamicEntry<'a>,
    /// Indicates the size of a symbol entry
    syment: &'a DynamicEntry<'a>,
    /// Indicates the start of the string table
    strtab: &'a DynamicEntry<'a>,
}

impl Syms<'_> {
    pub fn nth(&self, n: usize) -> Result<(Sym, &str), PixieError> {
        let sym: Sym = self.symtab.parse_nth(&self.syment, n)?;
        let name = unsafe { self.strtab.as_slice().as_ptr().add(sym.name as _).cstr() };
        Ok((sym, name))
    }

    /// Find a symbol by name. Will end up panicking if the symbol
    /// is not found!
    pub fn by_name(&self, name: &str) -> Result<Sym, PixieError> {
        let mut i = 0;
        loop {
            let (sym, sym_name) = self.nth(i)?;
            if sym_name == name {
                return Ok(sym);
            }
            i += 1;
        }
    }
}

/// Align *down* to the nearest 4K boundary
pub fn floor(val: u64) -> u64 {
    val & !0xFFF
}

/// Align *up* to the nearest 4K boundary
pub fn ceil(val: u64) -> u64 {
    if floor(val) == val {
        val
    } else {
        floor(val + 0x1000)
    }
}

/// Given a convex hull, align its start *down* to the nearest 4K boundary and
/// its end *up* to the nearest 4K boundary
pub fn align_hull(hull: Range<u64>) -> Range<u64> {
    floor(hull.start)..ceil(hull.end)
}
