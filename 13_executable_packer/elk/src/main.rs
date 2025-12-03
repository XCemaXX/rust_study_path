use std::{env, error::Error, fs};

use region::Protection;

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = env::args().nth(1).expect("usage: elk FILE");
    let input = fs::read(&input_path)?;
    println!("Analyzing {input_path:?}...");

    let file = match delf::File::parse_or_print_error(&input[..]) {
        Some(f) => f,
        None => std::process::exit(1),
    };
    println!("{file:#?}");

    let rela_entries = file.read_rela_entries().unwrap_or_else(|e| {
        println!("Could not read relocations: {e:?}");
        Default::default()
    });
    let base = 0x400000_usize;

    println!("Loading with base address @ 0x{base:x}");
    let non_empty_load_segments = file
        .program_headers
        .iter()
        .filter(|ph| ph.r#type == delf::SegmentType::Load)
        .filter(|ph| !ph.mem_range().is_empty());

    let mut mappings = Vec::new();
    for ph in non_empty_load_segments {
        let mem_range = ph.mem_range();
        println!("Mapping segment @ {mem_range:?} with {:?}", ph.flags);
        let len: usize = (mem_range.end - mem_range.start).into();

        let start: usize = mem_range.start.0 as usize + base;
        let aligned_start = align_lo(start);
        let padding = start - aligned_start;
        let len = len + padding;

        let addr: *mut u8 = unsafe { std::mem::transmute(aligned_start) };
        println!("Addr: {addr:p}, Padding: {padding:08x}");
        let map = mmap::MemoryMap::new(
            len,
            &[mmap::MapOption::MapWritable, mmap::MapOption::MapAddr(addr)],
        )?;
        unsafe {
            std::ptr::copy_nonoverlapping(ph.data.as_ptr(), addr.add(padding), len);
        }

        let mut num_relocs = 0;
        for reloc in &rela_entries {
            if !mem_range.contains(&reloc.offset) {
                continue;
            }
            num_relocs += 1;
            unsafe {
                let real_segment_start = addr.add(padding);
                let offset_into_segment = reloc.offset - mem_range.start;
                let reloc_addr = real_segment_start.add(offset_into_segment.into());

                match reloc.r#type {
                    delf::RelType::Relative => {
                        // this assumes `reloc_addr` is 8-byte aligned. if this isn't
                        // the case, we would crash, and so would the target executable
                        let reloc_addr: *mut u64 = std::mem::transmute(reloc_addr);
                        let reloc_value = reloc.addend + delf::Addr(base as u64);
                        std::ptr::write_unaligned(reloc_addr, reloc_value.0);
                    }
                    t => panic!("Unsupported relocation type {t:?}"),
                }
            }
        }
        if num_relocs > 0 {
            println!("(Applied {} relocations)", num_relocs);
        }

        let mut protection = Protection::NONE;
        for flag in ph.flags.iter() {
            protection |= match flag {
                delf::SegmentFlag::Execute => region::Protection::EXECUTE,
                delf::SegmentFlag::Write => region::Protection::WRITE,
                delf::SegmentFlag::Read => region::Protection::READ,
            }
        }
        unsafe {
            region::protect(addr, len, protection)?;
        }
        mappings.push(map);
    }

    unsafe {
        jmp(std::mem::transmute(file.entry_point.0 as usize + base));
    }
    Ok(())
}

unsafe fn jmp(addr: *const u8) {
    let fn_ptr: fn() = unsafe { std::mem::transmute(addr) };
    fn_ptr();
}

fn align_lo(x: usize) -> usize {
    x & !0xFFF
}
