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

    println!("Disassembling {input_path:?}...");
    let code_ph = file
        .program_headers
        .iter()
        .find(|ph| ph.mem_range().contains(&file.entry_point))
        .expect("segment with entry point not found");
    // ndisasm(&code_ph.data[..], file.entry_point)?;

    // picked by fair 4KiB-aligned dice roll
    let base = 0x400000_usize;
    println!("Mapping {:?} in memory...", input_path);
    let mut mappings = Vec::new();

    for ph in file
        .program_headers
        .iter()
        .filter(|ph| ph.r#type == delf::SegmentType::Load)
        .filter(|ph| !ph.mem_range().is_empty())
    {
        let mem_range = ph.mem_range();
        println!("Mapping segment @ {mem_range:?} with {:?}", ph.flags);
        let len: usize = (mem_range.end - mem_range.start).into();
        let start: usize = mem_range.start.0 as usize + base;
        let aligned_start = align_lo(start);
        let padding = start - aligned_start;
        let len = len + padding;

        let addr: *mut u8 = aligned_start as _;
        println!("Addr: {addr:p}, Padding: {padding:08x}");
        let map = mmap::MemoryMap::new(
            len,
            &[mmap::MapOption::MapWritable, mmap::MapOption::MapAddr(addr)],
        )?;
        println!("Copyiing segment data...");
        {
            let dst = unsafe { std::slice::from_raw_parts_mut(addr.add(padding), ph.data.len()) };
            dst.copy_from_slice(&ph.data[..]);
        }
        println!("Adjusting permissions...");
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
    pause("jmp")?;
    unsafe {
        jmp((file.entry_point.0 as usize + base) as _);
    }

    /*println!("Executing {:?} in memory...", input_path);
    let code = &code_ph.data;
    pause("protect")?;
    unsafe {
        region::protect(
            code.as_ptr(),
            code.len(),
            region::Protection::READ_WRITE_EXECUTE,
        )?;
    }
    let entry_offset = file.entry_point - code_ph.vaddr;
    let entry_point = unsafe { code.as_ptr().add(entry_offset.into()) };
    println!("       code  @ {:?}", code.as_ptr());
    println!("entry offset @ {entry_offset:?}");
    println!("entry point  @ {entry_point:?}");

    pause("jmp")?;
    unsafe {
        jmp(entry_point);
    }*/

    Ok(())
}

fn ndisasm(code: &[u8], origin: delf::Addr) -> Result<(), Box<dyn Error>> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };

    let mut child = Command::new("ndisasm")
        .arg("-b")
        .arg("64")
        .arg("-o")
        .arg(format!("{}", origin.0))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child.stdin.as_mut().unwrap().write_all(code)?;
    let output = child.wait_with_output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

unsafe fn jmp(addr: *const u8) {
    let fn_ptr: fn() = unsafe { std::mem::transmute(addr) };
    fn_ptr();
}

fn pause(reason: &str) -> Result<(), Box<dyn Error>> {
    println!("Press enter to {reason}...");
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    Ok(())
}

fn align_lo(x: usize) -> usize {
    x & !0xFFF
}
