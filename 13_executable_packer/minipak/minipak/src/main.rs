#![no_std]
#![no_main]

use core::arch::naked_asm;

extern crate encore;

use encore::prelude::*;

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn _start() -> ! {
    naked_asm!("mov rdi, rsp", "call pre_main", "ud2")
}

#[unsafe(no_mangle)]
unsafe extern "C" fn pre_main(_stack_top: *mut u8) -> ! {
    unsafe {
        init_allocator();
        main().unwrap();
        syscall::exit(0);
    }
}

fn main() -> Result<(), EncoreEror> {
    {
        let file = File::open("/etc/os-release")?;
        let map = file.map()?;
        let s = core::str::from_utf8(&map[..]).unwrap();
        for l in s.lines() {
            println!("> {}", l);
        }
    }
    {
        let file = File::open("/lib64/ld-linux-x86-64.so.2")?;
        let map = file.map()?;
        let s = core::str::from_utf8(&map[1..4]).unwrap();
        println!("{}", s);
    }

    Ok(())
}
