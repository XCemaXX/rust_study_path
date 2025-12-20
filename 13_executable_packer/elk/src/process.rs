use std::{
    cmp::{max, min},
    collections::HashMap,
    ffi::CString,
    io::Read,
    ops::Range,
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::name::Name;
use derive_more::Debug;
use enumflags2::BitFlags;
use mmap::{MapOption, MemoryMap};
use multimap::MultiMap;

pub struct Process<S: ProcessState> {
    pub state: S,
}

pub trait ProcessState {
    fn loader(&self) -> &Loader;
}

pub struct Loader {
    pub objects: Vec<Object>,
    pub objects_by_path: HashMap<PathBuf, usize>,
    pub search_path: Vec<PathBuf>,
}

pub struct Loading {
    pub loader: Loader,
}

impl ProcessState for Loading {
    fn loader(&self) -> &Loader {
        &self.loader
    }
}

impl Process<Loading> {
    pub fn new() -> Self {
        Self {
            state: Loading {
                loader: Loader {
                    objects: Vec::new(),
                    objects_by_path: HashMap::new(),
                    search_path: vec!["/usr/lib".into(), "/usr/lib/x86_64-linux-gnu/".into()],
                },
            },
        }
    }

    pub fn load_object<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, LoadError> {
        let loader = &mut self.state.loader;
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
        loader.search_path.extend(
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
        let mem_mmap = std::mem::ManuallyDrop::new(MemoryMap::new(
            mem_size,
            &[MapOption::MapReadable, MapOption::MapWritable],
        )?);
        let base = delf::Addr(mem_mmap.data() as _) - mem_range.start;

        let segments = load_segments()
            .map(|ph| -> Result<_, LoadError> {
                let vaddr = delf::Addr(ph.vaddr.0 & !0xFFF);
                let padding = ph.vaddr - vaddr;
                let offset = ph.offset - padding;
                let filesz = ph.filesz + padding;

                //print!("Mapping {ph:#?}");
                //println!(
                //    " | with offset {offset:#?}, vaddr {vaddr:#?}, base {base:#?}, filesz {filesz:?}",
                //    base = base + vaddr
                //);
                let options = &[
                    MapOption::MapReadable,
                    MapOption::MapWritable,
                    MapOption::MapExecutable,
                    MapOption::MapFd(fs_file.as_raw_fd()),
                    MapOption::MapOffset(offset.into()),
                    MapOption::MapAddr((base + vaddr).as_ptr()),
                ];
                let map = MemoryMap::new(filesz.into(), options)?;

                if ph.memsz > ph.filesz {
                    let mut zero_start = base + ph.mem_range().start + ph.filesz;
                    let zero_len = ph.memsz - ph.filesz;
                    unsafe {
                        zero_start
                            .as_mut_slice(zero_len.into())
                            .iter_mut()
                            .for_each(|i| *i = 0_u8);
                    }
                }

                Ok(Segment {
                    map: map.into(),
                    vaddr_range: vaddr..(ph.vaddr + ph.memsz),
                    padding,
                    flags: ph.flags,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let syms = file.read_dynsym_entries()?;
        let syms: Vec<_> = if syms.is_empty() {
            Vec::new()
        } else {
            let dynstr = file
                .get_dynamic_entry(delf::DynamicTag::StrTab)
                .unwrap_or_else(|_| panic!("String table not found in {path:?}"));
            let segment = segments
                .iter()
                .find(|seg| seg.vaddr_range.contains(&dynstr))
                .unwrap_or_else(|| panic!("Segment not found for string table in {path:#?}"));

            syms.into_iter()
                .map(|sym| {
                    let name = Name::mapped(
                        segment.map.clone(),
                        (dynstr + sym.name - segment.vaddr_range.start).into(),
                    );
                    NamedSym { sym, name }
                })
                .collect::<Vec<_>>()
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
            rels,
        };

        let index = loader.objects.len();
        loader.objects.push(object);
        loader.objects_by_path.insert(path, index);
        Ok(index)
    }

    pub fn object_path(&self, name: &str) -> Result<PathBuf, LoadError> {
        let loader = &self.state.loader;
        loader
            .search_path
            .iter()
            .filter_map(|prefix| prefix.join(name).canonicalize().ok())
            .find(|path| path.exists())
            .ok_or_else(|| LoadError::NotFound(name.into()))
    }

    pub fn get_object(&mut self, name: &str) -> Result<GetResult, LoadError> {
        let path = self.object_path(name)?;
        self.state
            .loader
            .objects_by_path
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
                .map(|index| &self.state.loader.objects[index].file)
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

    pub fn allocate_tls(mut self) -> Process<TlsAllocated> {
        let mut offsets = HashMap::new();
        let mut storage_space = 0;
        for obj in &mut self.state.loader.objects {
            let needed = obj
                .file
                .segment_of_type(delf::SegmentType::TLS)
                .map(|ph| ph.memsz.0)
                .unwrap_or_default() as u64;

            if needed == 0 {
                continue;
            }
            storage_space += needed;
            let offset = delf::Addr(storage_space);
            offsets.insert(obj.base, offset);
        }

        let storage_space = storage_space as usize;
        let tcbhead_size = 704;
        let total_size = storage_space + tcbhead_size;

        let mut block = Vec::with_capacity(total_size);
        let tcb_addr = delf::Addr(block.as_ptr() as u64 + storage_space as u64);
        for _ in 0..storage_space {
            block.push(0_u8);
        }

        let _tcb = block.extend(&tcb_addr.0.to_le_bytes());
        let _dtv = block.extend(&0_u64.to_le_bytes());
        let _thread_pointer = block.extend(&tcb_addr.0.to_le_bytes());
        let _multiple_threads = block.extend(&0_u32.to_le_bytes());
        let _gscope_flag = block.extend(&0_u32.to_le_bytes());
        let _sysinfo = block.extend(&0_u64.to_le_bytes());
        let _stack_guard = block.extend(&0xDEADBEEF_u64.to_le_bytes());
        let _pointer_guard = block.extend(&0xFEEDFACE_u64.to_le_bytes());
        while block.len() < block.capacity() {
            block.push(0_u8);
        }

        let tls = Tls {
            offsets,
            block,
            tcb_addr,
        };

        Process {
            state: TlsAllocated {
                loader: self.state.loader,
                tls,
            },
        }
    }
}

pub struct TlsAllocated {
    loader: Loader,
    pub tls: Tls,
}

impl ProcessState for TlsAllocated {
    fn loader(&self) -> &Loader {
        &self.loader
    }
}

pub struct Relocated {
    loader: Loader,
    tls: Tls,
}

impl ProcessState for Relocated {
    fn loader(&self) -> &Loader {
        &self.loader
    }
}

impl Process<TlsAllocated> {
    fn apply_relocation(&self, objrel: ObjectRel) -> Result<(), RelocationError> {
        use delf::RelType as RT;
        let ObjectRel { obj, rel } = objrel;
        let reltype = rel.r#type;
        let addend = rel.addend;

        let wanted = ObjectSym {
            obj,
            sym: &obj.syms[rel.sym as usize],
        };

        let ignore_self = matches!(reltype, RT::Copy);

        let found = if rel.sym == 0 {
            ResolvedSym::Undefined
        } else {
            match self.lookup_symbol(&wanted, ignore_self) {
                undef @ ResolvedSym::Undefined => match wanted.sym.sym.bind {
                    delf::SymBind::Weak => undef,
                    _ => return Err(RelocationError::UndefinedSymbol(wanted.sym.clone())),
                },
                defined => defined,
            }
        };

        match reltype {
            RT::_64 => unsafe {
                objrel.addr().set(found.value() + addend);
            },
            RT::Relative => unsafe {
                objrel.addr().set(obj.base + addend);
            },
            RT::IRelative => unsafe {
                type Selector = unsafe extern "C" fn() -> delf::Addr;
                let selector: Selector = std::mem::transmute(obj.base + addend);
                objrel.addr().set(selector());
            },
            RT::Copy => unsafe {
                objrel.addr().write(found.value().as_slice(found.size()));
            },
            RT::GlobDat | RT::JumpSlot => unsafe {
                objrel.addr().set(found.value());
            },
            RT::TPOff64 => unsafe {
                if let ResolvedSym::Defined(sym) = found {
                    let obj_offset =
                        self.state
                            .tls
                            .offsets
                            .get(&sym.obj.base)
                            .unwrap_or_else(|| {
                                panic!(
                                    "No thread-local storage allocated for object {:?}",
                                    sym.obj.file
                                )
                            });
                    let obj_offset = -(obj_offset.0 as i64);
                    let offset =
                        obj_offset + sym.sym.sym.value.0 as i64 + objrel.rel.addend.0 as i64;
                    objrel.addr().set(offset);
                }
            },
            RT::DTPMOD64 | RT::DTPOFF64 => {
                // TODO. Implement rela
                println!("skip for now relocation {reltype:?}")
            }
            _ => {
                return Err(RelocationError::UnimplementedRelocation(
                    obj.path.clone(),
                    reltype,
                ));
            }
        }
        Ok(())
    }

    pub fn apply_relocations(self) -> Result<Process<Relocated>, RelocationError> {
        let rels = self
            .state
            .loader
            .objects
            .iter()
            .rev()
            .flat_map(|obj| obj.rels.iter().map(move |rel| ObjectRel { obj, rel }));
        for rel in rels {
            self.apply_relocation(rel)?;
        }
        let res = Process {
            state: Relocated {
                loader: self.state.loader,
                tls: self.state.tls,
            },
        };
        Ok(res)
    }
}

pub struct TlsInitialized {
    loader: Loader,
    tls: Tls,
}

impl ProcessState for TlsInitialized {
    fn loader(&self) -> &Loader {
        &self.loader
    }
}

impl Process<Relocated> {
    pub fn initialize_tls(self) -> Process<TlsInitialized> {
        let tls = &self.state.tls;

        for obj in &self.state.loader.objects {
            let Some(ph) = obj.file.segment_of_type(delf::SegmentType::TLS) else {
                continue;
            };
            if let Some(offset) = tls.offsets.get(&obj.base).cloned() {
                unsafe {
                    (tls.tcb_addr - offset).write((ph.vaddr + obj.base).as_slice(ph.filesz.into()));
                }
            }
        }

        Process {
            state: TlsInitialized {
                loader: self.state.loader,
                tls: self.state.tls,
            },
        }
    }
}

pub struct Protected {
    loader: Loader,
    tls: Tls,
}

impl ProcessState for Protected {
    fn loader(&self) -> &Loader {
        &self.loader
    }
}

impl Process<TlsInitialized> {
    pub fn adjust_protections(self) -> Result<Process<Protected>, region::Error> {
        for obj in &self.state.loader.objects {
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
        let res = Process {
            state: Protected {
                loader: self.state.loader,
                tls: self.state.tls,
            },
        };
        Ok(res)
    }
}

impl Process<Protected> {
    fn build_stack(opts: &StartOptions) -> Vec<u64> {
        let mut stack = Vec::new();
        let null = 0_u64;

        macro_rules! push {
            ($x:expr) => {
                stack.push($x as u64)
            };
        }

        let argc = opts.args.len();
        push!(argc);

        for argv in &opts.args {
            push!(argv.as_ptr());
        }
        push!(null);

        for envp in &opts.env {
            push!(envp.as_ptr());
        }
        push!(null);

        for auxv in &opts.auxv {
            push!(auxv.typ);
            push!(auxv.value);
        }
        push!(AuxType::Null);
        push!(null);

        if stack.len() % 2 == 1 {
            push!(null);
        }

        stack
    }

    pub fn start(self, opts: &StartOptions) -> ! {
        let exec = &self.state.loader.objects[opts.exec_index];
        let entry_point = exec.file.entry_point + exec.base;
        let stack = Self::build_stack(opts);

        unsafe {
            set_fs(self.state.tls.tcb_addr.0);
            jmp(entry_point.as_ptr(), stack.as_ptr(), stack.len())
        }
    }
}

impl<S: ProcessState> Process<S> {
    fn lookup_symbol(&self, wanted: &ObjectSym, ignore_self: bool) -> ResolvedSym<'_> {
        let loader = self.state.loader();
        let candidates = loader
            .objects
            .iter()
            .filter(|&obj| !(ignore_self && std::ptr::eq(wanted.obj, obj)));

        for obj in candidates {
            if let Some(sym) = obj
                .sym_map
                .get_vec(&wanted.sym.name)
                .into_iter()
                .flatten()
                .find(|sym| !sym.sym.shndx.is_undef())
            {
                return ResolvedSym::Defined(ObjectSym { obj, sym });
            }
        }
        ResolvedSym::Undefined
    }
}

#[inline(never)]
unsafe fn jmp(entry_point: *const u8, stack_contents: *const u64, qword_count: usize) -> ! {
    unsafe {
        core::arch::asm!(
            // allocate (qword_count * 8) bytes
            "mov {tmp}, {qword_count}",
            "sal {tmp}, 3",
            "sub rsp, {tmp}",

            "2:",
            // start at i = (n-1)
            "sub {qword_count}, 1",
            // copy qwords to the stack
            "mov {tmp}, QWORD PTR [{stack_contents}+{qword_count}*8]",
            "mov QWORD PTR [rsp+{qword_count}*8], {tmp}",
            // loop if i isn't zero, break otherwise
            "test {qword_count}, {qword_count}",
            "jnz 2b",

            "jmp {entry_point}",

            entry_point = in(reg) entry_point,
            stack_contents = in(reg) stack_contents,
            qword_count = in(reg) qword_count,
            tmp = out(reg) _,
        );
        core::arch::asm!("ud2", options(noreturn));
    }
}

#[inline(never)]
unsafe fn set_fs(addr: u64) {
    const SYSCALL: u64 = 158;
    const ARCH_SET_FS: u64 = 0x1002;

    unsafe {
        core::arch::asm!(
            "syscall",
            inout("rax") SYSCALL => _,
            in("rdi") ARCH_SET_FS,
            in("rsi") addr,
            lateout("rcx") _, lateout("r11") _
        )
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
    pub rels: Vec<delf::Rela>,
}

#[derive(Debug, Clone)]
struct ObjectSym<'a> {
    obj: &'a Object,
    sym: &'a NamedSym,
}

impl ObjectSym<'_> {
    fn value(&self) -> delf::Addr {
        self.obj.base + self.sym.sym.value
    }
}

#[derive(Debug)]
enum ResolvedSym<'a> {
    Defined(ObjectSym<'a>),
    Undefined,
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
    rel: &'a delf::Rela,
}

impl ObjectRel<'_> {
    fn addr(&self) -> delf::Addr {
        self.obj.base + self.rel.offset
    }
}

#[derive(Debug)]
pub struct Segment {
    #[debug(skip)]
    pub map: Arc<MemoryMap>,
    pub vaddr_range: Range<delf::Addr>,
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
    #[error("{0:?}: unimplemented relocation: {1:?}")]
    UnimplementedRelocation(PathBuf, delf::RelType),
    #[allow(unused)]
    #[error("unknown symbol number: {0}")]
    UnknownSymbolNumber(u32),
    #[error("undifined symbol: {0:?}")]
    UndefinedSymbol(NamedSym),
}

#[derive(Debug, Clone)]
pub struct NamedSym {
    sym: delf::Sym,
    name: Name,
}

pub struct StartOptions {
    pub exec_index: usize,
    pub args: Vec<CString>,
    pub env: Vec<CString>,
    pub auxv: Vec<Auxv>,
}

pub struct Auxv {
    typ: AuxType,
    value: u64,
}

impl Auxv {
    // can be replaced with strum
    const KNOWN_TYPES: &'static [AuxType] = &[
        AuxType::ExecFd,
        AuxType::PHdr,
        AuxType::PhEnt,
        AuxType::PhNum,
        AuxType::PageSz,
        AuxType::Base,
        AuxType::Flags,
        AuxType::Entry,
        AuxType::NotElf,
        AuxType::Uid,
        AuxType::EUid,
        AuxType::Gid,
        AuxType::EGid,
        AuxType::Platform,
        AuxType::HwCap,
        AuxType::ClkTck,
        AuxType::Secure,
        AuxType::BasePlatform,
        AuxType::Random,
        AuxType::HwCap2,
        AuxType::RseqFeatureSize,
        AuxType::RseqAlign,
        AuxType::ExecFn,
        AuxType::SysInfo,
        AuxType::SysInfoEHdr,
        AuxType::MinSigStkSz,
    ];

    pub fn get(typ: AuxType) -> Option<Self> {
        unsafe extern "C" {
            // from libc
            fn getauxval(typ: u64) -> u64;
        }

        unsafe {
            match getauxval(typ as u64) {
                0 => None,
                value => Some(Self { typ, value }),
            }
        }
    }

    pub fn get_known() -> Vec<Self> {
        Self::KNOWN_TYPES
            .iter()
            .copied()
            .filter_map(Self::get)
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum AuxType {
    /// End of vector
    Null = 0,
    /// Entry should be ignored
    _Ignore = 1,
    /// File descriptor of program
    ExecFd = 2,
    /// Program headers for program
    PHdr = 3,
    /// Size of program header entry
    PhEnt = 4,
    /// Number of program headers
    PhNum = 5,
    /// System page size
    PageSz = 6,
    /// Base address of interpreter
    Base = 7,
    /// Flags
    Flags = 8,
    /// Entry point of program
    Entry = 9,
    /// Program is not ELF
    NotElf = 10,
    /// Real uid
    Uid = 11,
    /// Effective uid
    EUid = 12,
    /// Real gid
    Gid = 13,
    /// Effective gid
    EGid = 14,
    /// String identifying CPU for optimizations
    Platform = 15,
    /// Arch-dependent hints at CPU capabilities
    HwCap = 16,
    /// Frequency at which times() increments
    ClkTck = 17,
    /// Secure mode boolean
    Secure = 23,
    /// String identifying real platform, may differ from Platform
    BasePlatform = 24,
    /// Address of 16 random bytes
    Random = 25,
    /// Extension of HwCap
    HwCap2 = 26,
    /// Rseq supported feature size
    RseqFeatureSize = 27,
    /// Rseq allocation alignment
    RseqAlign = 28,
    /// Filename of program
    ExecFn = 31,

    SysInfo = 32,
    SysInfoEHdr = 33,
    /// Minimal stack size for signal delivery
    MinSigStkSz = 51,
}

#[derive(Debug)]
pub struct Tls {
    offsets: HashMap<delf::Addr, delf::Addr>,
    #[allow(unused)]
    block: Vec<u8>,
    tcb_addr: delf::Addr,
}
