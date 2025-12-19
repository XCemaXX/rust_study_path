use core::arch::asm;

pub const STDOUT_FILENO: u32 = 1;

pub unsafe fn write(fd: u32, buf: *const u8, count: usize) {
    let syscall_number: u64 = 1;
    unsafe {
        asm!(
            "syscall",
            inout("rax") syscall_number => _,
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            lateout("rcx") _, lateout("r11") _,
            options(nostack)
        )
    }
}

pub unsafe fn strlen(mut s: *const u8) -> usize {
    let mut count = 0;
    unsafe {
        while *s != b'\0' {
            count += 1;
            s = s.add(1);
        }
    }
    count
}

pub unsafe fn exit(code: i32) -> ! {
    let syscall_number: u64 = 60;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall_number,
            in("rdi") code,
            options(noreturn)
        )
    }
}

pub fn print_str(s: &[u8]) {
    unsafe {
        write(STDOUT_FILENO, s.as_ptr(), s.len());
    }
}

pub fn print_num(mut n: usize) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();

    if n == 0 {
        print_str(b"0");
        return;
    }
    while n != 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n = n / 10;
    }
    print_str(&buf[i..]);
}

pub fn print_hex(mut n: usize) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();

    print_str(b"0x");
    if n == 0 {
        print_str(b"0");
        return;
    }
    while n != 0 {
        i -= 1;
        let u = (n % 16) as u8;
        let c = match u {
            0..=9 => b'0' + u,
            _ => b'a' + u - 10,
        };
        buf[i] = c;
        n = n / 16;
    }
    print_str(&buf[i..]);
}

pub enum PrintArg<'a> {
    String(&'a [u8]),
    Number(usize),
    Hex(usize),
}

impl<'a> From<usize> for PrintArg<'a> {
    fn from(v: usize) -> Self {
        PrintArg::Number(v)
    }
}

impl<'a> From<&'a [u8]> for PrintArg<'a> {
    fn from(v: &'a [u8]) -> Self {
        PrintArg::String(v)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for PrintArg<'a> {
    fn from(v: &'a [u8; N]) -> Self {
        PrintArg::String(v.as_ref())
    }
}

pub fn print(args: &[PrintArg]) {
    for arg in args {
        match arg {
            PrintArg::String(s) => print_str(s),
            PrintArg::Number(n) => print_num(*n),
            PrintArg::Hex(n) => print_hex(*n),
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:expr),+) => {
        print(&[
            $($arg.into()),+
        ])
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:expr),+) => {
        print!($($arg),+,b"\n")
    };
}
