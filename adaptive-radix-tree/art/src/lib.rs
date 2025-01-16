use tree::ArtInner;

pub(crate) mod node;
pub(crate) mod tree;

pub struct Art {
    inner: ArtInner,
}

impl Art {
    pub fn new() -> Self {
        Self {
            inner: ArtInner::new(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.inner.get(key)
    }
}
