use std::fmt;

use crate::parse;
use derive_more::{Add, Sub};
use nom::{Parser as _, combinator, number::complete::le_u64};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, Sub)]
pub struct Addr(pub u64);

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<u64> for Addr {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Into<usize> for Addr {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Addr {
    /// # Safety
    ///
    /// This can create dangling pointers
    pub unsafe fn as_ptr<T>(&self) -> *const T {
        unsafe { std::mem::transmute(self.0 as usize) }
    }

    /// # Safety
    ///
    /// This can create dangling pointers
    pub unsafe fn as_mut_ptr<T>(&self) -> *mut T {
        unsafe { std::mem::transmute(self.0 as usize) }
    }

    pub unsafe fn as_slice<T>(&self, len: usize) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), len) }
    }

    pub unsafe fn as_mut_slice<T>(&mut self, len: usize) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), len) }
    }

    pub unsafe fn write(&self, src: &[u8]) {
        unsafe {
            std::ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), src.len());
        }
    }

    pub unsafe fn set<T>(&self, src: T) {
        unsafe {
            std::ptr::write_unaligned(self.as_mut_ptr(), src);
        }
    }

    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        combinator::map(le_u64, From::from).parse(i)
    }
}
