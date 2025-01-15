use inner::ArtInner;

pub(crate) mod inner;
pub(crate) mod iter;
pub(crate) mod node;

pub struct Art {
    inner: ArtInner,
}

impl Art {
    pub fn new() -> Self {
        Self {
            inner: ArtInner::new(),
        }
    }

    pub fn insert(&mut self, key: &[u8], value: &[u8]) {
        self.inner.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.inner.get(key)
    }
}
