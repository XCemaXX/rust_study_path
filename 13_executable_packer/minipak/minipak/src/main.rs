#![no_std]
#![no_main]

use core::arch::naked_asm;

extern crate encore;

use encore::prelude::*;
use pixie::{EndMarker, Manifest, PixieError, Resource, Writer};

mod cli;

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

const PAGE_SIZE: u64 = 4 * 1024;

fn main(env: Env) -> Result<(), PixieError> {
    let args = cli::Args::parse(&env);

    let mut output = Writer::new(&args.output, 0o755)?;

    {
        let stage1 = include_bytes!(concat!(
            env!("OUT_DIR"),
            "/embeds/x86_64-unknown-linux-gnu/embed/stage1"
        ));
        output.write_all(stage1)?;
    }

    let guest_offset = output.offset();
    let guest_compressed_len;
    let guest_len;
    {
        let guest = File::open(&args.input)?;
        let guest = guest.map()?;
        let guest = guest.as_ref();
        guest_len = guest.len();
        let compressed = lz4_flex::compress_prepend_size(guest);
        guest_compressed_len = compressed.len();
        output.write_all(&compressed)?;
    }

    output.align(PAGE_SIZE)?;
    let manifest_offest = output.offset();
    {
        let manifest = Manifest {
            guest: Resource {
                offset: guest_offset as _,
                len: guest_compressed_len as _,
            },
        };
        output.write_deku(&manifest)?;
    }
    {
        let marker = EndMarker {
            manifest_offset: manifest_offest as _,
        };
        output.write_deku(&marker)?;
    }

    println!(
        "Wrote {} ({:.2}% of input)",
        args.output,
        output.offset() as f64 / guest_len as f64 * 100.
    );

    Ok(())
}
