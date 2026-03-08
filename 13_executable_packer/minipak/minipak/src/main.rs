#![no_std]
#![no_main]

use core::{arch::naked_asm, ops::Range};

extern crate encore;

use encore::prelude::*;
use error::Error;
use pixie::{
    align_hull, ElfClass, ElfMachine, ElfType, EndMarker, Endianness, Manifest, MappedObject,
    Object, ObjectHeader, OsAbi, ProgramHeader, Resource, SegmentType, Writer,
};

mod cli;
mod error;

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

fn main(env: Env) -> Result<(), Error> {
    let args = cli::Args::parse(&env);

    println!("Packing quest {:?} into {:?}", args.input, args.output);
    let guest = File::open(args.input)?;
    let guest_map = guest.map()?;
    let guest = guest_map.as_ref();
    let guest = Object::new(guest)?;

    let guest_hull = guest.segments().load_convex_hull()?;
    let mut output = Writer::new(args.output, 0o755)?;
    relink_stage1(guest_hull, &mut output)?;

    let stage2_slice = include_bytes!(concat!(
        env!("OUT_DIR"),
        "/embeds/x86_64-unknown-linux-gnu/embed/libstage2.so"
    ));
    let stage2_offset = output.offset();
    println!("Copying stage2 at 0x{stage2_offset:x}");
    output.write_all(stage2_slice)?;
    output.align(0x8)?;

    println!("Compressing guest...");
    let compressed_guest = lz4_flex::compress_prepend_size(guest_map.as_ref());
    let guest_offset = output.offset();
    println!("copying compressed quest at 0x{guest_offset:x}");
    output.write_all(&compressed_guest)?;
    output.align(0x8)?;

    let manifest_offset = output.offset();
    println!("Writing manifest at 0x{manifest_offset:x}");
    let manifest = Manifest {
        stage2: Resource {
            offset: stage2_offset as _,
            len: stage2_slice.len(),
        },
        guest: Resource {
            offset: guest_offset as _,
            len: compressed_guest.len(),
        },
    };
    output.write_deku(&manifest)?;
    output.align(0x8)?;

    println!("Writing end marker");
    let end_marker = EndMarker {
        manifest_offset: manifest_offset as _,
    };
    output.write_deku(&end_marker)?;

    println!("Written to ({})", args.output);

    Ok(())
}

fn relink_stage1(guest_hull: Range<u64>, writer: &mut Writer) -> Result<(), Error> {
    let obj = Object::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/embeds/x86_64-unknown-linux-gnu/embed/libstage1.so"
    )))?;
    let hull = obj.segments().load_convex_hull()?;
    assert_eq!(hull.start, 0, "stage1 must be relocatable");

    let base_offset = if guest_hull.start == 0 {
        0x800000
    } else {
        guest_hull.start
    };
    println!("Picked base_offset 0x{base_offset:x}");
    let hull = (hull.start + base_offset)..(hull.end + base_offset);
    println!("Stage1 hull: {hull:x?}");
    println!(" Guest hull: {guest_hull:x?}");

    let mut mapped = MappedObject::new(&obj, None)?;
    println!("Loaded stage1");

    mapped.relocate(base_offset)?;
    println!("Relocated stage1");

    println!("Looking for `entry` in stage1...");
    let entry_sym = mapped.lookup_sym("entry")?;
    let entry_point = base_offset + entry_sym.value;

    let mut load_segs = obj
        .segments()
        .of_type(SegmentType::Load)
        .collect::<Vec<_>>();

    let out_header = ObjectHeader {
        class: ElfClass::Elf64,
        endianness: Endianness::Little,
        version: 1,
        os_abi: OsAbi::NetBsd,
        typ: ElfType::Exec,
        machine: ElfMachine::X86_64,
        version_bits: 1,
        entry_point,

        flags: 0,
        hdr_size: ObjectHeader::SIZE,
        // Two additional segments: one for `brk` alignment, and GNU_STACK.
        ph_count: load_segs.len() as u16 + 2,
        ph_offset: ObjectHeader::SIZE as _,
        ph_entsize: ProgramHeader::SIZE,
        sh_offset: 0,
        sh_entsize: 0,
        sh_count: 0,
        sh_nidx: 0,
    };
    writer.write_deku(&out_header)?;

    // We keep stage1's original PT_LOAD file offsets, only shifting vaddr/paddr by `base_offset`.
    // This keeps the ELF invariant `p_offset % p_align == p_vaddr % p_align` intact.
    //
    // Modern toolchains often produce a PT_LOAD at file offset 0 (vaddr 0) containing important
    // data (.dynsym/.rela.dyn/.rodata/etc). Our output ELF header + program headers also live at
    // the beginning of the file, so when copying that first segment we must avoid overwriting the
    // headers. We handle that in the copy loop below by skipping the prefix up to `copy_start_offset`.
    for seg in &load_segs {
        let mut ph = seg.header().clone();
        ph.vaddr += base_offset;
        ph.paddr += base_offset;
        writer.write_deku(&ph)?;
    }

    // Insert dummy segment to offset the `brk` to its original position
    // for the guest, if we can.
    {
        let current_hull = align_hull(hull);
        let desired_hull = align_hull(guest_hull);

        let pad_size = if current_hull.end > desired_hull.end {
            println!("WARNING: Guest executable is too small, the `brk` will be wrong.");
            println!(" {current_hull:x?} {desired_hull:x?}");
            0x0
        } else {
            desired_hull.end - current_hull.end
        };

        let ph = ProgramHeader {
            typ: SegmentType::Load,
            flags: ProgramHeader::WRITE | ProgramHeader::READ,
            offset: 0,
            vaddr: current_hull.end,
            paddr: current_hull.end,
            filesz: 0,
            memsz: pad_size,
            align: 0x1000,
        };
        writer.write_deku(&ph)?;
    }

    // Add a GNU_STACK program header for alignment and to make it
    // non-executable.
    {
        let ph = ProgramHeader {
            typ: SegmentType::GnuStack,
            flags: ProgramHeader::WRITE | ProgramHeader::READ,
            offset: 0,
            vaddr: 0,
            paddr: 0,
            filesz: 0,
            memsz: 0,
            align: 0x10,
        };
        writer.write_deku(&ph)?;
    }

    // Sort load segments by file offset and copy them.
    {
        load_segs.sort_by_key(|&seg| seg.header().offset);

        println!("Copying stage1 segments...");
        let copy_start_offset = writer.offset();

        // If stage1 has a PT_LOAD at file offset 0 (common for modern toolchains),
        // we rely on being able to skip only a small prefix (our new ELF header/PHDR)
        // and still copy the remaining tail of that segment.
        //
        // If the skipped prefix fully covers the segment, stage1 will miss critical
        // data (.dynsym/.rela*/.rodata) and will crash very early. In that case we
        // must fall back to repacking PT_LOAD offsets (the more complex path).
        if let Some(zero) = load_segs.iter().find(|seg| seg.header().offset == 0) {
            let filesz = zero.header().filesz;
            assert!(
                copy_start_offset < filesz,
                "stage1 first PT_LOAD starts at offset 0 but is too small to survive relinking: \
                copy_start_offset=0x{copy_start_offset:x} >= filesz=0x{filesz:x}. \
                Need to repack PT_LOAD offsets (the more complex path)."
            );
        }

        // Copy `[copy_offset..ph.offset+filesz)` from this segment.
        // Usually only the first PT_LOAD has `ph.offset == 0`, so this neatly avoids
        // clobbering the output ELF header/program headers.
        println!("copy_start_offset = 0x{:x}", copy_start_offset);
        let copied_segments = load_segs.into_iter().filter_map(|seg| {
            let ph = seg.header();

            // Start copying either from the segment's file offset
            // or from copy_start_offset if the segment overlaps the new ELF header.
            let copy_offset = ph.offset.max(copy_start_offset);

            // If the entire segment is covered by the skipped prefix, there is
            // nothing to copy.
            if copy_offset >= ph.offset + ph.filesz {
                return None;
            }
            println!("copying {ph:?}");
            Some((ph, copy_offset))
        });

        for (ph, copy_offset) in copied_segments {
            // Pad space up to where we actually start copying
            writer.pad(copy_offset - writer.offset())?;

            // Translate file offset -> virtual address inside this PT_LOAD:
            let delta = copy_offset - ph.offset;
            let start = ph.vaddr + delta;
            let len = ph.filesz - delta;
            let end = start + len;

            // `mapped` contains stage1 mapped at base 0 with relocations applied,
            // so `vaddr_slice` expects original (pre-base_offset) virtual addresses.
            writer.write_all(mapped.vaddr_slice(start..end))?;
        }
    }
    // Pad end of last segment with zeros:
    writer.align(0x1000)?;
    Ok(())
}
