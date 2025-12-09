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
        let file = delf::File::parse_or_print_error(&input[..])
            .ok_or_else(|| LoadError::ParseError(path.clone()))?;

        let origin = path
            .parent()
            .ok_or_else(|| LoadError::InvalidPath(path.clone()))?
            .to_str()
            .ok_or_else(|| LoadError::InvalidPath(path.clone()))?;
        self.search_path.extend(
            file.dynamic_entry_strings(delf::DynamicTag::RunPath)
                .map(|path| path.replace("$ORIGIN", &origin))
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
        let mem_mmap = std::mem::ManuallyDrop::new(MemoryMap::new(mem_size, &[])?);
        let base = delf::Addr(mem_mmap.data() as _) - mem_range.start;

        let segments = load_segments()
            .map(|ph| {
                let vaddr = delf::Addr(ph.vaddr.0 & !0xFFF);
                let padding = ph.vaddr - vaddr;
                let offset = ph.offset - padding;
                let memsz = ph.memsz + padding;

                print!("Mapping {ph:#?}");
                println!(
                    " | with offset {offset:#?}, vaddr {vaddr:#?}, base {base:#?}, memsz {memsz:?}",
                    base = base + vaddr
                );
                let options = &[
                    MapOption::MapReadable,
                    MapOption::MapWritable,
                    MapOption::MapFd(fs_file.as_raw_fd()),
                    MapOption::MapOffset(offset.into()),
                    MapOption::MapAddr(unsafe { (base + vaddr).as_ptr() }),
                ];
                MemoryMap::new(memsz.into(), options).map(|map| Segment {
                    map,
                    padding,
                    flags: ph.flags,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let syms = file.read_syms()?;

        let object = Object {
            path: path.clone(),
            base,
            segments,
            file,
            mem_range,
            syms,
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

    pub fn apply_relocations(&self) -> Result<(), RelocationError> {
        for obj in self.objects.iter().rev() {
            let rela_entries = obj.file.read_rela_entries();

            let rels = match rela_entries {
                Ok(rels) => rels,
                Err(e) => {
                    println!("Skip relocations for {bin:?}: {e:?}", bin = obj.path);
                    continue;
                }
            };
            println!(
                "Applying relocations for {:?}. Found {} rela entries",
                obj.path,
                rels.len()
            );

            for reloc in rels {
                println!("Found {:?}", reloc);
                match reloc.r#type {
                    delf::RelType::Known(delf::KnownRelType::_64) => {
                        let name = obj.sym_name(reloc.sym)?;
                        println!("Looking up {name:?}");
                        let (lib, sym) = self
                            .lookup_symbol(&name, None)?
                            .ok_or(RelocationError::UndefinedSymbol(name))?;

                        let offset = obj.base + reloc.offset;
                        let value = sym.value + lib.base + reloc.addend;
                        println!("Found at {:?}:{:?} in {:?}", sym.value, value, lib.path,);

                        unsafe {
                            let ptr: *mut u64 = offset.as_mut_ptr();
                            println!("Applying reloc@ {ptr:?}");
                            // fixes *ptr = value.0;
                            std::ptr::write_unaligned(ptr, value.0);
                        }
                    }
                    delf::RelType::Known(delf::KnownRelType::Copy) => {
                        let name = obj.sym_name(reloc.sym)?;
                        let (lib, sym) = self
                            .lookup_symbol(&name, Some(obj))?
                            .ok_or(RelocationError::UndefinedSymbol(name.clone()))?;
                        println!(
                            "Found at {:?} at {:?} (size {:?}) in {:?}",
                            name, sym.value, sym.size, lib.path,
                        );
                        unsafe {
                            let src = (sym.value + lib.base).as_ptr();
                            let dst = (reloc.offset + obj.base).as_mut_ptr();
                            std::ptr::copy_nonoverlapping::<u8>(src, dst, sym.size as usize);
                        }
                    }
                    delf::RelType::Known(t) => {
                        return Err(RelocationError::UnimplementedRelocation(t));
                    }
                    delf::RelType::Unknown(num) => {
                        return Err(RelocationError::UnknownRelocation(num));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn lookup_symbol(
        &self,
        name: &str,
        ignore: Option<&Object>,
    ) -> Result<Option<(&Object, &delf::Sym)>, RelocationError> {
        let candidates = self
            .objects
            .iter()
            .filter(|&obj| ignore.map_or(true, |ignored| !std::ptr::eq(obj, ignored)));

        for obj in candidates {
            for (i, sym) in obj.syms.iter().enumerate() {
                if obj.sym_name(i as u32)? == name {
                    return Ok(Some((obj, sym)));
                }
            }
        }
        Ok(None)
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

#[allow(unused)]
#[derive(Debug)]
pub struct Object {
    pub path: PathBuf,
    pub base: delf::Addr,
    #[debug(skip)]
    pub file: delf::File,
    #[debug(skip)]
    pub segments: Vec<Segment>,
    pub mem_range: Range<delf::Addr>,
    #[debug(skip)]
    pub syms: Vec<delf::Sym>,
}

impl Object {
    pub fn sym_name(&self, index: u32) -> Result<String, RelocationError> {
        self.file
            .get_string(self.syms[index as usize].name)
            .map_err(|_| RelocationError::UnknownSymbolNumber(index))
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Segment {
    #[debug(skip)]
    pub map: MemoryMap,
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
}

#[allow(unused)]
pub enum GetResult {
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
    #[error("unknown relocation: {0}")]
    UnknownRelocation(u32),
    #[error("unimplemented relocation: {0:?}")]
    UnimplementedRelocation(delf::KnownRelType),
    #[error("unknown symbol number: {0}")]
    UnknownSymbolNumber(u32),
    #[error("undifined symbol: {0}")]
    UndefinedSymbol(String),
}

fn _dump_maps(msg: &str) {
    println!("======== MEMORY MAPS: {}", msg);
    std::fs::read_to_string(format!("/proc/{pid}/maps", pid = std::process::id()))
        .unwrap()
        .lines()
        .filter(|line| line.contains("hello-dl") || line.contains("libmsg.so"))
        .for_each(|line| println!("{line}"));
    println!("=============================");
}
