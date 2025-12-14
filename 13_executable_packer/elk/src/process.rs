use std::{
    cmp::{max, min},
    collections::HashMap,
    io::Read,
    ops::Range,
    os::fd::AsRawFd,
    path::{Path, PathBuf},
};

use derive_more::Debug;
use enumflags2::BitFlags;
use mmap::{MapOption, MemoryMap};
use crate::name::Name;
use multimap::MultiMap;

#[derive(Debug)]
pub struct Process {
    pub objects: Vec<Object>,
    pub objects_by_path: HashMap<PathBuf, usize>,
    pub search_path: Vec<PathBuf>,
}

impl Process {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            objects_by_path: HashMap::new(),
            search_path: vec!["/usr/lib".into(), "/usr/lib/x86_64-linux-gnu/".into()],
        }
    }

    pub fn load_object<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, LoadError> {
        let path = path
            .as_ref()
            .canonicalize()
            .map_err(|e| LoadError::IO(path.as_ref().to_path_buf(), e))?;

        let mut fs_file = std::fs::File::open(&path).map_err(|e| LoadError::IO(path.clone(), e))?;
        let mut input = Vec::new();
        fs_file
            .read_to_end(&mut input)
            .map_err(|e| LoadError::IO(path.clone(), e))?;

        println!("loading {path:?}");
        let file = delf::File::parse_or_print_error(input)
            .ok_or_else(|| LoadError::ParseError(path.clone()))?;

        let origin = path
            .parent()
            .ok_or_else(|| LoadError::InvalidPath(path.clone()))?
            .to_str()
            .ok_or_else(|| LoadError::InvalidPath(path.clone()))?;
        self.search_path.extend(
            file.dynamic_entry_strings(delf::DynamicTag::RunPath)
                .map(|path| String::from_utf8_lossy(path))
                .map(|path| path.replace("$ORIGIN", origin))
                .inspect(|path| println!("Found RunPath entry {path:?}"))
                .map(PathBuf::from),
        );

        let load_segments = || {
            file.program_headers
                .iter()
                .filter(|ph| ph.r#type == delf::SegmentType::Load)
                .filter(|ph| !ph.mem_range().is_empty())
        };

        let mem_range = load_segments()
            .map(|ph| ph.mem_range())
            .fold(None, |acc, range| match acc {
                None => Some(range),
                Some(acc) => Some(convex_hull(acc, range)),
            })
            .ok_or(LoadError::NoLoadSegments)?;

        let mem_size: usize = (mem_range.end - mem_range.start).into();
        let mem_mmap = std::mem::ManuallyDrop::new(MemoryMap::new(mem_size, &[MapOption::MapReadable, MapOption::MapWritable])?);
        let base = delf::Addr(mem_mmap.data() as _) - mem_range.start;

        let segments = load_segments()
            .map(|ph| -> Result<_, LoadError> {
                let vaddr = delf::Addr(ph.vaddr.0 & !0xFFF);
                let padding = ph.vaddr - vaddr;
                let offset = ph.offset - padding;
                let filesz = ph.filesz + padding;

                print!("Mapping {ph:#?}");
                println!(
                    " | with offset {offset:#?}, vaddr {vaddr:#?}, base {base:#?}, filesz {filesz:?}",
                    base = base + vaddr
                );
                let options = &[
                    MapOption::MapReadable,
                    MapOption::MapWritable,
                    MapOption::MapExecutable,
                    MapOption::MapFd(fs_file.as_raw_fd()),
                    MapOption::MapOffset(offset.into()),
                    MapOption::MapAddr(unsafe { (base + vaddr).as_ptr() }),
                ];
                let map = MemoryMap::new(filesz.into(), options)?;
                
                if ph.memsz > ph.filesz {
                    let mut zero_start = base + ph.mem_range().start + ph.filesz;
                    let zero_len = ph.memsz -ph.filesz;
                    unsafe {
                        zero_start.as_mut_slice(zero_len.into()).iter_mut().for_each(|i| *i = 0_u8);
                    }
                }
                
                Ok(Segment {
                    map,
                    padding,
                    flags: ph.flags,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let syms = file.read_dynsym_entries()?;
        let syms: Vec<_> = if syms.is_empty() {
            Vec::new()
        } else {
            let strtab = file.get_dynamic_entry(delf::DynamicTag::StrTab)
            .unwrap_or_else(|_| panic!("String table not found in {path:?}"));
            syms.into_iter().map(|sym| unsafe {
                let name = Name::from_addr(base + strtab + sym.name);
                NamedSym {sym, name}
            }).collect::<Vec<_>>()
        };

        let sym_map = MultiMap::from_iter(syms.iter().cloned().map(|sym| (sym.name.clone(), sym)));

        let mut rels = Vec::new();
        rels.extend(file.read_rela_entries()?);
        rels.extend(file.read_jmp_rel_entries()?);

        let object = Object {
            path: path.clone(),
            base,
            segments,
            file,
            mem_range,
            syms,
            sym_map,
            rels
        };

        let index = self.objects.len();
        self.objects.push(object);
        self.objects_by_path.insert(path, index);
        Ok(index)
    }

    pub fn object_path(&self, name: &str) -> Result<PathBuf, LoadError> {
        self.search_path
            .iter()
            .filter_map(|prefix| prefix.join(name).canonicalize().ok())
            .find(|path| path.exists())
            .ok_or_else(|| LoadError::NotFound(name.into()))
    }

    pub fn get_object(&mut self, name: &str) -> Result<GetResult, LoadError> {
        let path = self.object_path(name)?;
        self.objects_by_path
            .get(&path)
            .map(|&index| Ok(GetResult::Cached(index)))
            .unwrap_or_else(|| self.load_object(path).map(GetResult::Fresh))
    }

    pub fn load_object_and_dependencies<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<usize, LoadError> {
        let index = self.load_object(path)?;

        let mut current = vec![index];
        while !current.is_empty() {
            current = current
                .into_iter()
                .map(|index| &self.objects[index].file)
                .flat_map(|file| file.dynamic_entry_strings(delf::DynamicTag::Needed))
                .map(|s| String::from_utf8_lossy(s).to_string())
                .collect::<Vec<_>>()
                .into_iter()
                .map(|dep| self.get_object(&dep))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter_map(GetResult::fresh)
                .collect();
        }
        Ok(index)
    }

    fn apply_relocation(&self, objrel: ObjectRel) -> Result<(), RelocationError> {
        use delf::RelType as RT;
        let ObjectRel { obj, rel } = objrel;
        let reltype = rel.r#type;
        let addend = rel.addend;

        let wanted = ObjectSym {
            obj,
            sym: &obj.syms[rel.sym as usize]
        };

        let ignore_self = matches!(reltype, RT::Copy);

        let found = if rel.sym == 0 {
            ResolvedSym::Undefined
        } else {
            match self.lookup_symbol(&wanted, ignore_self) {
                undef @ ResolvedSym::Undefined => {
                    match wanted.sym.sym.bind {
                        delf::SymBind::Weak => undef,
                        _ => return Err(RelocationError::UndefinedSymbol(format!("{wanted:?}"))),
                    }
                },
                defined => defined,
            }
        };

        match reltype {
            RT::_64 => unsafe{
                objrel.addr().set(found.value() + addend);
            },
            RT::Relative => unsafe {
                objrel.addr().set(obj.base + addend);
            }
            RT::IRelative => unsafe {
                let selector: extern "C" fn() -> delf::Addr =
                    std::mem::transmute(obj.base + addend);
                objrel.addr().set(selector());
            }
            RT::Copy => unsafe {
                objrel.addr().write(found.value().as_slice(found.size()));
            },
            _ => {
                return Err(RelocationError::UnimplementedRelocation(reltype));
            }
        }
        Ok(())

    }

    pub fn apply_relocations(&self) -> Result<(), RelocationError> {
        let rels = self.objects.iter().rev().flat_map(|obj| obj.rels.iter().map(move |rel| ObjectRel{obj, rel}));
        for rel in rels {
            self.apply_relocation(rel)?;
        }
        Ok(())
    }

    fn lookup_symbol(
        &self,
        wanted: &ObjectSym,
        ignore_self: bool,
    ) -> ResolvedSym<'_> {
        let candidates = self
            .objects
            .iter()
            .filter(|&obj| !(ignore_self && std::ptr::eq(wanted.obj, obj)));

        for obj in candidates {
            if let Some(sym) = obj.sym_map.get_vec(&wanted.sym.name)
                .into_iter().flatten()
                .find(|sym| !sym.sym.shndx.is_undef()) {
                return ResolvedSym::Defined(ObjectSym { obj, sym });
            }
        }
        ResolvedSym::Undefined
    }

    pub fn adjust_protections(&self) -> Result<(), region::Error> {
        for obj in &self.objects {
            for seg in &obj.segments {
                let mut protection = region::Protection::NONE;
                for flag in seg.flags.iter() {
                    protection |= match flag {
                        delf::SegmentFlag::Execute => region::Protection::EXECUTE,
                        delf::SegmentFlag::Write => region::Protection::WRITE,
                        delf::SegmentFlag::Read => region::Protection::READ,
                    }
                }
                unsafe {
                    region::protect(seg.map.data(), seg.map.len(), protection)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Object {
    #[allow(unused)]
    pub path: PathBuf,
    pub base: delf::Addr,
    #[debug(skip)]
    pub file: delf::File<Vec<u8>>,
    #[debug(skip)]
    pub segments: Vec<Segment>,
    #[allow(unused)]
    pub mem_range: Range<delf::Addr>,
    #[debug(skip)]
    syms: Vec<NamedSym>,
    #[debug(skip)]
    sym_map: MultiMap<Name, NamedSym>,
    #[debug(skip)]
    pub rels: Vec<delf::Rela>
}

#[derive(Debug, Clone)]
struct ObjectSym<'a> {
    obj: &'a Object,
    sym: &'a NamedSym
}

impl ObjectSym<'_> {
    fn value(&self) -> delf::Addr {
        self.obj.base + self.sym.sym.value
    }
}

#[derive(Debug)]
enum ResolvedSym<'a> {
    Defined(ObjectSym<'a>),
    Undefined
}

impl ResolvedSym<'_> {
    fn value(&self) -> delf::Addr {
        match self {
            ResolvedSym::Defined(s) => s.value(),
            ResolvedSym::Undefined => delf::Addr(0x0),
        }
    }

    fn size(&self) -> usize {
        match self {
            ResolvedSym::Defined(s) => s.sym.sym.size as usize,
            ResolvedSym::Undefined => 0,
        }
    }
}

#[derive(Debug)]
struct ObjectRel<'a> {
    obj: &'a Object,
    rel: &'a delf::Rela
}

impl ObjectRel<'_> {
    fn addr(&self) -> delf::Addr {
        self.obj.base + self.rel.offset
    }
}

#[derive(Debug)]
pub struct Segment {
    #[debug(skip)]
    pub map: MemoryMap,
    #[allow(unused)]
    pub padding: delf::Addr,
    pub flags: BitFlags<delf::SegmentFlag>,
}

#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("ELF object not found: {0}")]
    NotFound(String),
    #[error("An invalid or unsupported path was encountered")]
    InvalidPath(PathBuf),
    #[error("I/O error on: {0}: {1}")]
    IO(PathBuf, std::io::Error),
    #[error("ELF object could not be parsed: {0}")]
    ParseError(PathBuf),
    #[error("ELF object has no load segments")]
    NoLoadSegments,
    #[error("ELF object could not be mapped in memory: {0}")]
    MapError(#[from] mmap::MapError),
    #[error("Could not read symbols from ELF object: {0}")]
    ReadSymsError(#[from] delf::ReadSymsError),
    #[error("Could not read relocations from ELF object: {0}")]
    ReadRelaError(#[from] delf::ReadRelaError),
}

pub enum GetResult {
    #[allow(unused)]
    Cached(usize),
    Fresh(usize),
}

impl GetResult {
    fn fresh(self) -> Option<usize> {
        if let Self::Fresh(index) = self {
            Some(index)
        } else {
            None
        }
    }
}

fn convex_hull(a: Range<delf::Addr>, b: Range<delf::Addr>) -> Range<delf::Addr> {
    (min(a.start, b.start))..max(a.end, b.end)
}

#[derive(thiserror::Error, Debug)]
pub enum RelocationError {
    #[error("unimplemented relocation: {0:?}")]
    UnimplementedRelocation(delf::RelType),
    #[allow(unused)]
    #[error("unknown symbol number: {0}")]
    UnknownSymbolNumber(u32),
    #[error("undifined symbol: {0}")]
    UndefinedSymbol(String),
}

#[derive(Debug, Clone)]
struct NamedSym {
    sym: delf::Sym,
    name: Name
}