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
    fake_ctype: Box<FakeCtype>,
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
                    fake_ctype: FakeCtype::new(),
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

        let mut initializers = Vec::new();
        if let Some(init) = file.dynamic_entry(delf::DynamicTag::Init) {
            let init = init + base;
            initializers.push(init);
        }

        if let Some(init_array) = file.dynamic_entry(delf::DynamicTag::InitArray) {
            if let Some(init_array_sz) = file.dynamic_entry(delf::DynamicTag::InitArraySz) {
                let init_array = base + init_array;
                let n = init_array_sz.0 as usize / std::mem::size_of::<delf::Addr>();

                let inits: &[delf::Addr] = unsafe { init_array.as_slice(n) };
                initializers.extend(inits.iter().map(|&init| init + base));
            }
        }

        let object = Object {
            path: path.clone(),
            base,
            segments,
            file,
            mem_range,
            syms,
            sym_map,
            rels,
            initializers,
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

        // TODO. Maybe need to improve tls, dtv...
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

    pub fn pathc_libc(&self) {
        let mut stub_map = HashMap::<&str, Vec<u8>>::new();

        // No such internal symbol in newer glibc
        stub_map.insert(
            "_dl_addr",
            vec![
                0x48, 0x31, 0xc0, // xor rax, rax
                0xc3, // ret
            ],
        );

        // Newer glibc exports dladdr/dladdr1 from libc (ptmalloc_init uses dl_addr internally).
        stub_map.insert(
            "dladdr",
            vec![
                0x48, 0x31, 0xc0, // xor rax, rax
                0xc3, // ret
            ],
        );
        stub_map.insert(
            "dladdr1",
            vec![
                0x48, 0x31, 0xc0, // xor rax, rax
                0xc3, // ret
            ],
        );

        // Minimal hack: bypass locale initialization (otherwise glibc hits TLS/TSD assumptions
        // we don't fully implement yet, and crashes in read_alias_file).
        stub_map.insert(
            "setlocale",
            vec![
                0x48, 0x31, 0xc0, // xor rax, rax
                0xc3, // ret
            ],
        );

        stub_map.insert(
            "exit",
            vec![
                0x48, 0x31, 0xff, // xor rdi, rdi
                0xb8, 0x3c, 0x00, 0x00, 0x00, // mov eax, 60
                0x0f, 0x05, // syscall
            ],
        );
        let pattern = "/libc.so.6";
        let libc = match self
            .state
            .loader
            .objects
            .iter()
            .find(|&obj| obj.path.to_string_lossy().contains(pattern))
        {
            Some(x) => x,
            None => {
                println!("Warning: could not find libc to patch!");
                return;
            }
        };

        for (name, instructions) in stub_map {
            let name = Name::owned(name);
            let sym = match libc.sym_map.get(&name) {
                Some(sym) => ObjectSym { obj: libc, sym },
                None => {
                    println!("expected to find symbol {:?} in {:?}", name, libc.path);
                    continue;
                }
            };
            println!("Patching libc funtion {:?} ({:?})", sym.value(), name);
            unsafe {
                sym.value().write(&instructions);
            }
        }

        // `__ctype_*_loc()` must return T **. Redirect them to our own static tables
        // to keep ctype users working with locale support stubbed out.
        let ctype_table = &self.state.loader.fake_ctype;
        let ctype_fixups = [
            ("__ctype_b_loc", &ctype_table.b_ptr as *const _ as *const ()),
            (
                "__ctype_tolower_loc",
                &ctype_table.tolower_ptr as *const _ as *const (),
            ),
            (
                "__ctype_toupper_loc",
                &ctype_table.toupper_ptr as *const _ as *const (),
            ),
        ];

        fn make_ret_ptr_stub(ptr: *const ()) -> Vec<u8> {
            let addr = ptr as u64;
            let mut code = Vec::with_capacity(11);
            code.extend([0x48, 0xB8]); // movabs rax, imm64
            code.extend(addr.to_le_bytes());
            code.push(0xC3); // ret
            code
        }

        for (name, ret_ptr) in ctype_fixups {
            let name = Name::owned(name);
            let sym = match libc.sym_map.get(&name) {
                Some(sym) => ObjectSym { obj: libc, sym },
                None => continue,
            };
            let code = make_ret_ptr_stub(ret_ptr);
            println!(
                "Patching libc function {:?} ({:?}) -> return fake ctype ptr {:?}",
                sym.value(),
                name,
                delf::Addr(ret_ptr as u64),
            );
            unsafe {
                sym.value().write(&code);
            }
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
    fn bootstrap_rtld_global(&self) {
        // On modern glibc, `ld-linux` defines `_rtld_global`, which is normally
        // initialized very early during rtld startup.
        //
        // In `elk`, ld-linux is loaded as a regular shared object and its entrypoint
        // is never executed, so `_rtld_global` remains zero-initialized and libc
        // crashes extremely early (e.g. in `__libc_start_main_impl`).
        //
        // Minimal startup hack: bring `_rtld_global` into a minimally valid state
        // so libc can proceed past early initialization.
        let name_global = Name::owned("_rtld_global");

        fn find_defined<'a>(obj: &'a Object, name: &Name) -> Option<&'a NamedSym> {
            obj.sym_map
                .get_vec(name)?
                .iter()
                .find(|sym| !sym.sym.shndx.is_undef())
        }

        // Many objects reference `_rtld_global` as an undefined symbol (e.g. libc),
        // but only ld-linux actually defines it. We must locate the defining object.
        let Some((obj_base, global_sym)) = self.state.loader.objects.iter().find_map(|obj| {
            let global = find_defined(obj, &name_global)?;
            Some((obj.base, global))
        }) else {
            return;
        };

        if global_sym.sym.size == 0 {
            return;
        }
        let global_addr = obj_base + global_sym.sym.value;

        // On this glibc, libc treats `_rtld_global` as a handle to some deeper structure:
        // it loads `_rtld_global` from the GOT, then does `mov (%r15), %r14` and uses `r14`
        // as a base pointer. If the first qword is null, libc crashes immediately.
        //
        // Minimal hack: Point it back to itself to ensure a non-NULL base pointer.
        unsafe {
            let first: u64 = std::ptr::read_unaligned(global_addr.as_ptr());
            if first == 0 {
                global_addr.set(global_addr.0);
            }
        }

        // More minimal hardening for modern glibc startup:
        // `__libc_start_main_impl` may call function pointers derived from
        // `_rtld_global + 0xa0` / `_rtld_global + 0x108`.
        // Setting them to null makes libc take the "skip" paths.
        unsafe {
            (global_addr + delf::Addr(0xa0)).set::<u64>(0);
            (global_addr + delf::Addr(0x108)).set::<u64>(0);
        }
    }

    fn apply_relocation<'a>(
        &self,
        objrel: ObjectRel<'a>,
        group: RelocGroup,
    ) -> Result<Option<ObjectRel<'a>>, RelocationError> {
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
            obj.symzero()
        } else {
            match self.lookup_symbol(&wanted, ignore_self) {
                undef @ ResolvedSym::Undefined => match wanted.sym.sym.bind {
                    delf::SymBind::Weak => undef,
                    _ => return Err(RelocationError::UndefinedSymbol(wanted.sym.clone())),
                },
                defined => defined,
            }
        };

        if let RelocGroup::Direct = group {
            if reltype == RT::Relative || found.is_indirect() {
                return Ok(Some(objrel));
            }
        }

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
        Ok(None)
    }

    pub fn apply_relocations(self) -> Result<Process<Relocated>, RelocationError> {
        let mut rels: Vec<_> = self
            .state
            .loader
            .objects
            .iter()
            .rev()
            .flat_map(|obj| obj.rels.iter().map(move |rel| ObjectRel { obj, rel }))
            .collect();

        for &group in &[RelocGroup::Direct, RelocGroup::Indirect] {
            println!("Applying {:?} relocations ({} left)", group, rels.len());
            rels = rels
                .into_iter()
                .map(|objrel| self.apply_relocation(objrel, group))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter_map(|x| x)
                .collect();
        }

        // After relocations, initialize ld-linux internal global state enough for libc startup.
        self.bootstrap_rtld_global();

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
        let initializers = self.initializers();

        let argc = opts.args.len() as i32;
        let mut argv: Vec<_> = opts.args.iter().map(|x| x.as_ptr()).collect();
        argv.push(std::ptr::null());
        let mut envp: Vec<_> = opts.env.iter().map(|x| x.as_ptr()).collect();
        envp.push(std::ptr::null());

        unsafe {
            set_fs(self.state.tls.tcb_addr.0);
            #[allow(clippy::clippy::needless_range_loop)]
            for i in 0..initializers.len() {
                call_init(initializers[i].1, argc, argv.as_ptr(), envp.as_ptr());
            }
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

    fn initializers(&self) -> Vec<(&Object, delf::Addr)> {
        self.state
            .loader()
            .objects
            .iter()
            .rev()
            .flat_map(|obj| obj.initializers.iter().map(move |&init| (obj, init)))
            .collect()
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

#[inline(never)]
unsafe fn call_init(addr: delf::Addr, argc: i32, argv: *const *const i8, envp: *const *const i8) {
    let init: extern "C" fn(argc: i32, argv: *const *const i8, envp: *const *const i8) =
        unsafe { std::mem::transmute(addr.0) };
    init(argc, argv, envp);
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
    #[debug(skip)]
    pub initializers: Vec<delf::Addr>,
}

impl Object {
    fn symzero(&self) -> ResolvedSym<'_> {
        ResolvedSym::Defined(ObjectSym {
            obj: &self,
            sym: &self.syms[0],
        })
    }
}

#[derive(Debug, Clone)]
struct ObjectSym<'a> {
    obj: &'a Object,
    sym: &'a NamedSym,
}

impl ObjectSym<'_> {
    fn value(&self) -> delf::Addr {
        let addr = self.sym.sym.value + self.obj.base;
        match self.sym.sym.r#type {
            delf::SymType::IFunc => unsafe {
                let src: extern "C" fn() -> delf::Addr = std::mem::transmute(addr);
                src()
            },
            _ => addr,
        }
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

    fn is_indirect(&self) -> bool {
        match self {
            Self::Undefined => false,
            Self::Defined(sym) => matches!(sym.sym.sym.r#type, delf::SymType::IFunc),
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

#[derive(Clone, Copy, Debug)]
pub enum RelocGroup {
    Direct,
    Indirect,
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

#[repr(C)]
struct FakeCtype {
    b_table: [u16; 256],
    b_ptr: *const u16,

    tolower_table: [i32; 256],
    tolower_ptr: *const i32,

    toupper_table: [i32; 256],
    toupper_ptr: *const i32,
}

impl FakeCtype {
    fn new() -> Box<Self> {
        let mut c = Box::new(Self {
            b_table: [0; 256],
            b_ptr: std::ptr::null(),

            tolower_table: [0; 256],
            tolower_ptr: std::ptr::null(),

            toupper_table: [0; 256],
            toupper_ptr: std::ptr::null(),
        });

        // __ctype_b: всё printable ASCII
        const _ISPRINT: u16 = 0x40;
        for i in 0x20u8..=0x7eu8 {
            c.b_table[i as usize] = _ISPRINT;
        }

        // tolower / toupper — identity
        for i in 0..256 {
            c.tolower_table[i] = i as i32;
            c.toupper_table[i] = i as i32;
        }

        c.b_ptr = c.b_table.as_ptr();
        c.tolower_ptr = c.tolower_table.as_ptr();
        c.toupper_ptr = c.toupper_table.as_ptr();

        c
    }
}
