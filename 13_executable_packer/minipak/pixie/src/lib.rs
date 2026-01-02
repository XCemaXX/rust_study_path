#![no_std]

extern crate alloc;

use core::ops::Range;

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
        self.base_offset == 0
    }

    /// Returns the offset between the object's base and where we loaded it
    pub fn base_offset(&self) -> u64 {
        self.base_offset
    }

    /// Returns the base address for this executable
    pub fn base(&self) -> u64 {
        self.mem.as_ptr() as _
    }
}
