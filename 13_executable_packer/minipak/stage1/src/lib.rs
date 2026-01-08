#![no_std]

extern crate alloc;

use core::arch::naked_asm;

use encore::prelude::*;
use pixie::{Manifest, MappedObject, Object};

macro_rules! info {
    ($($tokens: tt)*) => {
        println!("[stage1] {}", alloc::format!($($tokens)*))
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn entry() -> ! {
    naked_asm!("mov rdi, rsp", "call pre_main", "ud2");
}

#[unsafe(no_mangle)]
#[inline(never)]
unsafe extern "C" fn pre_main(stack_top: *mut u8) -> ! {
    unsafe {
        init_allocator();
        main(stack_top);
        syscall::exit(0);
    }
}

#[inline(never)]
unsafe fn main(stack_top: *mut u8) {
    info!("Stack top: {:?}", stack_top);

    let file = File::open("/proc/self/exe").unwrap();
    let map = file.map().unwrap();
    let slice = map.as_ref();
    let manifest = Manifest::read_from_full_slice(slice).unwrap();

    let s2_slice = &slice[manifest.stage2.as_range()];
    let s2_obj = Object::new(s2_slice).unwrap();
    let mut s2_mapped = MappedObject::new(&s2_obj, None).unwrap();
    info!(
        "Mapped stage2 at base 0x{:x} (offset 0x{:x})",
        s2_mapped.base(),
        s2_mapped.base_offset()
    );
    info!("Relocating stage2...");
    s2_mapped.relocate(s2_mapped.base_offset()).unwrap();
    info!("Relocating stage2... done!");

    let s2_entry = s2_mapped.lookup_sym("entry").unwrap();
    info!("Found entry_sym {s2_entry:?}");
    unsafe {
        let entry: unsafe extern "C" fn(*mut u8) -> ! =
            core::mem::transmute(s2_mapped.base_offset() + s2_entry.value);
        entry(stack_top);
    }
}
