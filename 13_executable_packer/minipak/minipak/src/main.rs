#![no_std]
#![no_main]

use core::{arch::naked_asm, ops::Range};

extern crate encore;

use encore::prelude::*;
use error::Error;
use pixie::{
    ElfClass, ElfMachine, ElfType, EndMarker, Endianness, Manifest, MappedObject, Object,
    ObjectHeader, OsAbi, ProgramHeader, Resource, SegmentType, Writer, align_hull,
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
    let guest = File::open(&args.input)?;
    let guest_map = guest.map()?;
    let guest = guest_map.as_ref();
    let guest = Object::new(guest)?;

    let guest_hull = guest.segments().load_convex_hull()?;
    let mut output = Writer::new(&args.output, 0o755)?;
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

    let static_headers = load_segs.iter().map(|seg| {
        let mut ph = seg.header().clone();
        ph.vaddr += base_offset;
        ph.paddr += base_offset;
        ph
    });
    for ph in static_headers {
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
        println!("copy_start_offset = 0x{:x}", copy_start_offset);
        let copied_segments = load_segs
            .into_iter()
            .filter(move |seg| seg.header().offset > copy_start_offset);

        for cp_seg in copied_segments {
            let ph = cp_seg.header();
            println!("copying {ph:?}");

            // Pad space between segments with zeros:
            writer.pad(ph.offset - writer.offset())?;

            let start = ph.vaddr;
            let len = ph.filesz;
            let end = start + len;

            writer.write_all(mapped.vaddr_slice(start..end))?;
        }
    }
    // Pad end of last segment with zeros:
    writer.align(0x1000)?;
    Ok(())
}
