use alloc::vec::Vec;
use core::fmt;

use crate::utils::NullTerminated;

#[repr(C)]
pub struct Auxv {
    typ: AuxType,
    pub value: u64,
}

impl fmt::Debug for Auxv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AT_{:?} = 0x{:x}", self.typ, self.value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct AuxType(u64);

impl AuxType {
    /// End of vector
    pub const NULL: Self = Self(0);
    /// Program headers for program
    pub const PHDR: Self = Self(3);
    /// Number of program headers
    pub const PHNUM: Self = Self(5);
    /// Base address of interpreter
    pub const BASE: Self = Self(7);
    /// Entry point of program
    pub const ENTRY: Self = Self(9);
}

impl fmt::Debug for AuxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::NULL => "NULL",
            Self::PHDR => "PHDR",
            Self::PHNUM => "PHNUM",
            Self::BASE => "BASE",
            Self::ENTRY => "ENTRY",
            _ => "(UNKNOWN)",
        })
    }
}

#[derive(Default)]
pub struct Env {
    pub vectors: Vec<&'static mut Auxv>,
    pub args: Vec<&'static str>,
    pub vars: Vec<&'static str>,
}

impl Env {
    /// # Safety
    /// - `stack_top` must be valid, properly aligned, and point to initialized memory.
    /// - The memory must contain null-terminated arguments, environment variables, and
    ///   auxiliary vectors ending with `AuxType::NULL`.
    pub unsafe fn read(stack_top: *mut u8) -> Self {
        let mut ptr = stack_top as *mut u64;
        let mut env = Self::default();
        unsafe {
            // Read arguments
            ptr = ptr.add(1);
            while *ptr != 0 {
                let arg = (*ptr as *const u8).cstr();
                env.args.push(arg);
                ptr = ptr.add(1);
            }

            // Read variables
            ptr = ptr.add(1);
            while *ptr != 0 {
                let arg = (*ptr as *const u8).cstr();
                env.vars.push(arg);
                ptr = ptr.add(1);
            }

            // Read auxiliary vectors
            ptr = ptr.add(1);
            let mut ptr = ptr as *mut Auxv;
            while (*ptr).typ != AuxType::NULL {
                env.vectors.push(ptr.as_mut().unwrap());
                ptr = ptr.add(1);
            }
        }
        env
    }

    pub fn find_vector(&mut self, typ: AuxType) -> &mut Auxv {
        self.vectors
            .iter_mut()
            .find(|v| v.typ == typ)
            .unwrap_or_else(|| panic!("aux vector {typ:?} not found"))
    }
}
