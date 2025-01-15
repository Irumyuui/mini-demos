const PREFIX_SIZE: usize = 10;

#[derive(Debug)]
pub(crate) struct Prefix {
    prefix: [u8; PREFIX_SIZE],
    prefix_len: u8,
}

impl Default for Prefix {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            prefix_len: Default::default(),
        }
    }
}

impl Prefix {
    pub(crate) fn new(prefix: &[u8]) -> Self {
        assert!(prefix.len() <= PREFIX_SIZE);

        let mut prefix_arr = [0; PREFIX_SIZE];
        prefix_arr[..prefix.len()].copy_from_slice(prefix);
        Self {
            prefix: prefix_arr,
            prefix_len: prefix.len() as u8,
        }
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.prefix[..self.prefix_len as usize]
    }

    pub(crate) fn len(&self) -> usize {
        self.prefix_len as _
    }
}

#[derive(Debug)]
pub(crate) struct InternalNodeMeta {
    prefix: Prefix,
    pub(crate) num_children: u16,
}

impl InternalNodeMeta {
    pub(crate) fn new() -> Self {
        Self {
            prefix: Prefix::default(),
            num_children: 0,
        }
    }

    pub(crate) fn with_prefix(prefix: &[u8]) -> Self {
        Self {
            prefix: Prefix::new(prefix),
            num_children: 0,
        }
    }

    pub(crate) fn prefix_slice(&self) -> &[u8] {
        self.prefix.as_slice()
    }

    pub(crate) fn prefix(&self) -> &Prefix {
        &self.prefix
    }
}
