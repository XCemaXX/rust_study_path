#[derive(Clone)]
pub enum Name {
    FromAddr { addr: delf::Addr, len: usize },
    Owned(Vec<u8>),
}

impl Name {
    /// # Safety
    ///
    /// `addr` must point to a null-terminated string, otherwise it's an UB
    pub unsafe fn from_addr(addr: delf::Addr) -> Self {
        let len = unsafe {
            addr.as_slice::<u8>(2048)
                .iter()
                .position(|&c| c == 0)
                .expect("scanned 2048 bytes without finding null-terminator for name")
        };
        Self::FromAddr { addr, len }
    }

    #[allow(dead_code)]
    pub fn owned<T: Into<Vec<u8>>>(v: T) -> Self {
        Self::Owned(v.into())
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Name::FromAddr { addr, len } => unsafe { addr.as_slice(*len) },
            Name::Owned(v) => v,
        }
    }
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = self.as_slice();
        match std::str::from_utf8(slice) {
            Ok(s) => std::fmt::Display::fmt(s, f),
            Err(_) => std::fmt::Debug::fmt(slice, f),
        }
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_slice(), other.as_slice())
    }
}
impl Eq for Name {}

impl std::hash::Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(self.as_slice(), state)
    }
}
