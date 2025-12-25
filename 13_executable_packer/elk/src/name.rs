use std::{ops::Range, sync::Arc};

use mmap::MemoryMap;

#[derive(Clone)]
pub enum Name {
    Mmapped {
        map: Arc<MemoryMap>,
        range: Range<usize>,
    },
    Owned(Vec<u8>),
}

impl Name {
    pub fn mapped(map: Arc<MemoryMap>, offset: usize) -> Self {
        let len = map
            .as_slice()
            .iter()
            .skip(offset)
            .position(|&c| c == 0)
            .expect("scanned 2048 bytes without finding null-terminator for name");
        Self::Mmapped {
            map,
            range: offset..offset + len,
        }
    }

    pub fn owned<T: Into<Vec<u8>>>(v: T) -> Self {
        Self::Owned(v.into())
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Name::Mmapped { map, range } => &map.as_slice()[range.clone()],
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

trait MemoryMapExt {
    fn as_slice(&self) -> &[u8];
}

impl MemoryMapExt for MemoryMap {
    fn as_slice(&self) -> &[u8] {
        // # Safety
        // hope on MemoryMap
        unsafe { std::slice::from_raw_parts(self.data(), self.len()) }
    }
}
