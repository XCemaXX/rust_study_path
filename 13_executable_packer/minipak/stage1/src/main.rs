#![no_std]
#![no_main]

use core::arch::naked_asm;

extern crate alloc;
extern crate encore;

use encore::prelude::*;
use pixie::{Manifest, MappedObject, Object, PixieError};

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn _start() -> ! {
    naked_asm!("mov rdi, rsp", "call pre_main", "ud2");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn pre_main(stack_top: *mut u8) -> ! {
    unsafe {
        init_allocator();
        main(stack_top, Env::read(stack_top)).unwrap();
        syscall::exit(0);
    }
}

macro_rules! info {
    ($($tokens: tt)*) => {
        println!("[stage1] {}", alloc::format!($($tokens)*))
    }
}

fn main(stack_top: *mut u8, mut env: Env) -> Result<(), PixieError> {
    info!("Hello from stage1!");

    let host = File::open("/proc/self/exe")?;
    let host = host.map()?;
    let host = host.as_ref();
    let manifest = Manifest::read_from_full_slice(host)?;

    let guest_range = manifest.guest.as_range();
    info!("The guest is at {guest_range:x?}");

    let guest_slice = &host[guest_range];
    let uncompressed_guest =
        lz4_flex::decompress_size_prepended(guest_slice).expect("invalid lz4 payload");

    let guest_obj = Object::new(&uncompressed_guest)?;

    let guest_mapped = MappedObject::new(&guest_obj, None)?;
    info!("Mapped guest at 0x{:x}", guest_mapped.base());

    let at_phdr = env.find_vector(AuxType::PHDR);
    at_phdr.value = guest_mapped.base() + guest_obj.header().ph_offset;

    let at_phnum = env.find_vector(AuxType::PHNUM);
    at_phnum.value = guest_obj.header().ph_count as _;

    let at_entry = env.find_vector(AuxType::ENTRY);
    at_entry.value = guest_mapped.base_offset() + guest_obj.header().entry_point;

    let entry_point = guest_mapped.base() + guest_obj.header().entry_point;
    info!("Jumping to guest's entry point 0x{entry_point:x}");
    unsafe { pixie::launch(stack_top, entry_point) }
}
