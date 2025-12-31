#![no_std]
#![no_main]

use core::arch::{asm, naked_asm};

extern crate encore;

use encore::prelude::*;
use pixie::{Manifest, PixieError};

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn _start() -> ! {
    naked_asm!("mov rdi, rsp", "call pre_main", "ud2");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn pre_main(stack_top: *mut u8) -> ! {
    unsafe {
        init_allocator();
        main(Env::read(stack_top)).unwrap();
        syscall::exit(0);
    }
}

fn main(env: Env) -> Result<(), PixieError> {
    println!("Hello from stage1!");

    let host = File::open("/proc/self/exe")?;
    let host = host.map()?;
    let host = host.as_ref();
    let manifest = Manifest::read_from_full_slice(host)?;

    let guest_range = manifest.guest.as_range();
    println!("The guest is at {guest_range:x?}");

    let guest_slice = &host[guest_range];
    let uncompressed_guest =
        lz4_flex::decompress_size_prepended(guest_slice).expect("invalid lz4 payload");

    let tmp_path = "/tmp/minipack-guest";
    {
        let mut guest = File::create(tmp_path, 0o755)?;
        guest.write_all(&uncompressed_guest[..])?;
    }

    {
        extern crate alloc;

        let tmp_path_nullter = format!("{tmp_path}\0");
        let argv: Vec<*const u8> = env
            .args
            .iter()
            .copied()
            .map(str::as_ptr)
            .chain(core::iter::once(core::ptr::null()))
            .collect();
        let envp: Vec<*const u8> = env
            .vars
            .iter()
            .copied()
            .map(str::as_ptr)
            .chain(core::iter::once(core::ptr::null()))
            .collect();
        let syscall_execve = 59;
        unsafe {
            asm!(
                "syscall",
                in("rax") syscall_execve,
                in("rdi") tmp_path_nullter.as_ptr(),
                in("rsi") argv.as_ptr(),
                in("rdx") envp.as_ptr(),
                options(noreturn),
            )
        }
    }
}
