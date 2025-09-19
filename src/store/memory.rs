use super::*;

pub struct MemStore {
    pub data: Vec<Contact>,
}

impl MemStore {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn iter(&self) -> MemStoreIter<'_> {
        MemStoreIter {
            inner: &self.data,
            idx: 0,
        }
    }
}

pub struct MemStoreIter<'a> {
    inner: &'a [Contact],
    idx: usize,
}

impl<'a> Iterator for MemStoreIter<'a> {
    type Item = &'a Contact;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.inner.len() {
            return None;
        }
        let contact = &self.inner[self.idx];
        self.idx += 1;
        Some(contact)
    }
}
