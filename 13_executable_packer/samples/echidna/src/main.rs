#![no_std]
#![no_main]

use core::{arch::naked_asm, slice};

mod support;
use support::*;

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() {
    naked_asm!("mov rdi, rsp", "call main")
}

#[unsafe(no_mangle)]
pub fn main(stack_top: *const u8) {
    unsafe {
        let argc = *(stack_top as *const u64);
        let argv = stack_top.add(8) as *const *const u8;

        let args = slice::from_raw_parts(argv, argc as usize);

        println!(b"received ", argc as usize, b" arguments:");
        for &arg in args {
            let arg = slice::from_raw_parts(arg, strlen(arg));
            println!(b" - ", arg)
        }

        println!(b"env variables");
        let mut envp = argv.add(argc as usize + 1) as *const *const u8;
        let mut filtered = 0;
        while !(*envp).is_null() {
            let var = *envp;
            let var = slice::from_raw_parts(var, strlen(var));

            if is_envvar_allowed(var) {
                println!(b" - ", var);
            } else {
                filtered += 1;
            }
            envp = envp.add(1);
        }
        println!(b"(+ ", filtered, b" redacted env variables)");

        println!(b"auxiliary vectors:");
        let mut auxv = envp.add(1) as *const Auxv;
        let null_auxv = Auxv { typ: 0, val: 0 };

        while (*auxv) != null_auxv {
            println!(b" - ", (*auxv).name(), b": ", (*auxv).formatted_val());
            auxv = auxv.add(1);
        }

        exit(argc as _)
    }
}

fn is_envvar_allowed(var: &[u8]) -> bool {
    const ALLOWED_ENV_VARS: &'static [&[u8]] = &[b"USER=", b"SHELL=", b"LANG="];
    for prefix in ALLOWED_ENV_VARS {
        if var.starts_with(prefix) {
            return true;
        }
    }
    false
}

#[derive(PartialEq)]
struct Auxv {
    typ: u64,
    val: u64,
}

impl Auxv {
    fn name(&self) -> &[u8] {
        match self.typ {
            2 => b"AT_EXECFD",
            3 => b"AT_PHDR",
            4 => b"AT_PHENT",
            5 => b"AT_PHNUM",
            6 => b"AT_PAGESZ",
            7 => b"AT_BASE",
            8 => b"AT_FLAGS",
            9 => b"AT_ENTRY",
            11 => b"AT_UID",
            12 => b"AT_EUID",
            13 => b"AT_GID",
            14 => b"AT_EGID",
            15 => b"AT_PLATFORM",
            16 => b"AT_HWCAP",
            17 => b"AT_CLKTCK",
            23 => b"AT_SECURE",
            24 => b"AT_BASE_PLATFORM",
            25 => b"AT_RANDOM",
            26 => b"AT_HWCAP2",
            27 => b"AT_RSEQ_FEATURE_SIZE",
            28 => b"AT_RSEQ_ALIGN",
            31 => b"AT_EXECFN",
            32 => b"AT_SYSINFO",
            33 => b"AT_SYSINFO_EHDR",
            51 => b"AT_MINSIGSTKSZ", // https://elixir.bootlin.com/linux/v6.6.119/source/include/uapi/linux/auxvec.h
            _ => b"??",
        }
    }

    fn formatted_val(&self) -> PrintArg<'_> {
        match self.typ {
            3 | 7 | 9 | 16 | 25 | 26 | 33 => PrintArg::Hex(self.val as usize),
            31 | 15 => {
                let s = unsafe {
                    let ptr = self.val as *const u8;
                    core::slice::from_raw_parts(ptr, strlen(ptr))
                };
                PrintArg::String(s)
            }
            _ => PrintArg::Number(self.val as usize),
        }
    }
}

// instead of
//#![feature(lang_items)] and #[lang = "eh_personality"]
#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {
    // To properly build:
    // 1. rebuild the core module -Zbuild-std=core,compiler_builtins,panic_abort -Z build-std-features=panic_immediate_abort
    // 2. set rustflags = ["-Zunstable-options", "-Cpanic=immediate-abort"]
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
