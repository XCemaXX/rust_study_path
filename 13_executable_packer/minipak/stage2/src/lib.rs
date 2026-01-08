#![no_std]

extern crate alloc;

use encore::prelude::*;
use pixie::{Manifest, MappedObject, Object, ObjectHeader, SegmentType};

macro_rules! info {
    ($($tokens: tt)*) => {
        println!("[stage2] {}", alloc::format!($($tokens)*))
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
unsafe extern "C" fn entry(stack_top: *mut u8) -> ! {
    unsafe {
        init_allocator();
        main(stack_top);
        syscall::exit(0);
    }
}

#[inline(never)]
unsafe fn main(stack_top: *mut u8) {
    info!("Stack top: {:?}", stack_top);

    let mut stack = unsafe { Env::read(stack_top as _) };

    let file = File::open("/proc/self/exe").unwrap();
    info!("Mapping self...");
    let map = file.map().unwrap();
    info!("Mapping self... done");
    let slice = map.as_ref();
    let manifest = Manifest::read_from_full_slice(slice).unwrap();

    let compressed_guest = &slice[manifest.guest.as_range()];
    let guest = lz4_flex::decompress_size_prepended(compressed_guest).unwrap();
    let guest_obj = Object::new(&guest).unwrap();
    let guest_hull = guest_obj.segments().load_convex_hull().unwrap();

    let at = if guest_hull.start == 0 {
        // guest is relocatable, load it with the same base as ourselves
        let elf_header_address = stack.find_vector(AuxType::PHDR).value;
        let self_base = elf_header_address - ObjectHeader::SIZE as u64;
        Some(self_base)
    } else {
        // guest is non-relocatable, it'll be loaded at its preferred offset
        None
    };
    let base_offset = at.unwrap_or_default();

    let guest_mapped = MappedObject::new(&guest_obj, at).unwrap();
    info!("Mapped guest at 0x{:x}", guest_mapped.base());

    let at_phdr = stack.find_vector(AuxType::PHDR);
    at_phdr.value = guest_mapped.base() + guest_obj.header().ph_offset;

    let at_phnum = stack.find_vector(AuxType::PHNUM);
    at_phnum.value = guest_obj.header().ph_count as _;

    let at_entry = stack.find_vector(AuxType::ENTRY);
    at_entry.value = guest_mapped.base_offset() + guest_obj.header().entry_point;

    let Ok(interp) = guest_obj.segments().find(SegmentType::Interp) else {
        let entry_point = base_offset + guest_obj.header().entry_point;
        info!("Jumping to guest's entry point 0x{entry_point:x}");
        unsafe { pixie::launch(stack_top, entry_point) }
    };

    let interp = core::str::from_utf8(interp.slice()).unwrap();
    info!("Should load interpreter {interp}");

    let interp_file = File::open(interp).unwrap();
    let interp_map = interp_file.map().unwrap();
    let interp_obj = Object::new(&interp_map).unwrap();
    let interp_hull = interp_obj.segments().load_convex_hull().unwrap();
    if interp_hull.start != 0 {
        panic!("Expected interpreter to be relocatable");
    }

    let interp_mapped = MappedObject::new(&interp_obj, None).unwrap();

    let at_base = stack.find_vector(AuxType::BASE);
    at_base.value = interp_mapped.base();

    let entry_point = interp_mapped.base() + interp_obj.header().entry_point;
    info!("Jumping to interpreter's entry point 0x{entry_point:x}");
    unsafe { pixie::launch(stack_top, entry_point) }
}
