#![no_std]
#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(lang_items)]

extern crate alloc;

pub mod error;
pub mod fs;
pub mod items;
pub mod memmap;
pub mod prelude;
pub mod syscall;
pub mod utils;
