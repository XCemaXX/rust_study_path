use bitflags::*;
use core::arch::asm;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(pub u64);

impl FileDescriptor {
    pub const STDIN: Self = Self(0);
    pub const STDOUT: Self = Self(1);
    pub const STDERR: Self = Self(2);
}

/// # Safety
/// Calls into the kernel.
#[inline(always)]
pub unsafe fn write(fd: FileDescriptor, buf: *const u8, count: u64) -> u64 {
    let syscall_number: u64 = 1;
    let mut rax = syscall_number;

    unsafe {
        asm!(
            "syscall",
            inout("rax") rax,
            in("rdi") fd.0,
            in("rsi") buf,
            in("rdx") count,
            lateout("rcx") _, lateout("r11") _,
            options(nostack)
        );
    }
    rax
}

/// # Safety
/// Calls into the kernel.
#[inline(always)]
pub unsafe fn exit(code: i32) -> ! {
    let syscall_number: u64 = 60;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall_number,
            in("rdi") code,
            options(noreturn, nostack)
        )
    }
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MmapProt: u64 {
        const READ = 0x1;
        const WRITE = 0x2;
        const EXEC = 0x4;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MmapFlags: u64 {
        const PRIVATE = 0x02;
        const FIXED = 0x10;
        const ANONYMOUS = 0x20;
    }
}

/// # Safety
/// Calls into the kernel. May unmap running code.
/// One of the most unsafe syscalls in existence.
#[inline(always)]
pub unsafe fn mmap(
    addr: u64,
    len: u64,
    prot: MmapProt,
    flags: MmapFlags,
    fd: FileDescriptor,
    off: u64,
) -> u64 {
    let syscall_number: u64 = 9;
    let mut rax = syscall_number;
    unsafe {
        asm!("syscall",
            inout("rax") rax,
            in("rdi") addr,
            in("rsi") len,
            in("rdx") prot.bits(),
            in("r10") flags.bits(),
            in("r8") fd.0,
            in("r9") off,
            lateout("rcx") _, lateout("r11") _,
            options(nostack),
        );
    }
    rax
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OpenFlags: u64 {
        const RDONLY = 0o0;
        const RDWR = 0o2;
        const CREAT = 0o100;
        const TRUNC = 0o1000;
    }
}

/// # Safety
/// Calls into the kernel.
#[inline(always)]
pub unsafe fn open(filename: *const u8, flags: OpenFlags, mode: u64) -> FileDescriptor {
    let syscall_number: u64 = 2;
    let mut rax = syscall_number;
    unsafe {
        asm!("syscall",
            inout("rax") rax,
            in("rdi") filename,
            in("rsi") flags.bits(),
            in("rdx") mode,
            lateout("rcx") _, lateout("r11") _,
            options(nostack),
        );
    }
    FileDescriptor(rax)
}

#[repr(C)]
pub struct Stat {
    _unused1: [u8; 48],
    pub size: u64,
    _unused2: [u8; 88],
}

// Compile-time size check for the x86_64 Linux ABI.
// If this fails, `fstat` will overwrite the stack and cause UB.
const _FILE_STAT_SIZE: [(); 144] = [(); core::mem::size_of::<Stat>()];

/// # Safety
/// Calls into the kernel.
#[inline(always)]
pub unsafe fn fstat(fd: FileDescriptor, buf: *mut Stat) -> u64 {
    let syscall_number: u64 = 5;
    let mut rax = syscall_number;
    unsafe {
        asm!(
            "syscall",
            inout("rax") rax,
            in("rdi") fd.0,
            in("rsi") buf,
            lateout("rcx") _, lateout("r11") _,
            options(nostack),
        );
    }
    rax
}

/// # Safety
/// Calls into the kernel.
#[inline(always)]
pub unsafe fn close(fd: FileDescriptor) -> u64 {
    let syscall_number: u64 = 3;
    let mut rax = syscall_number;
    unsafe {
        asm!(
            "syscall",
            inout("rax") rax,
            in("rdi") fd.0,
            lateout("rcx") _, lateout("r11") _,
            options(nostack),
        );
    }
    rax
}

/// # Safety
/// Calls into the kernel.
#[inline(always)]
pub unsafe fn munmap<T>(addr: *const T, len: u64) -> u64 {
    let syscall_number: u64 = 11;
    let mut rax = syscall_number;
    unsafe {
        asm!(
            "syscall",
            inout("rax") rax,
            in("rdi") addr,
            in("rsi") len,
            lateout("rcx") _, lateout("r11") _,
            options(nostack),
        );
    }
    rax
}

/// # Safety
/// Calls into the kernel.
#[inline(always)]
pub unsafe fn dup(fd: u64) {
    // for debugging break point. Duplicate file descriptor
    let syscall_number: u64 = 32;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall_number,
            in("rdi") fd,
            lateout("rcx") _, lateout("r11") _,
            options(nostack),
        )
    }
}
