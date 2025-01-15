use std::ptr::NonNull;

pub(crate) const PREFIX_SIZE: usize = 10;
pub(crate) type Prefix = [u8; PREFIX_SIZE];

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) enum ArtNode {
    Internal(NonNull<ArtBaseNode>),
    Leaf(NonNull<ArtLeafNode>),
    None,
}

#[repr(C)]
pub(crate) struct ArtBaseNode {
    prefix: Prefix,
    prefix_len: u8,
    pub(crate) ch_num: u16,
    pub(crate) ty: ArtNodeType,
}

#[repr(C)]
pub(crate) struct ArtNode4 {
    pub(crate) base: ArtBaseNode,
    pub(crate) keys: [u8; 4],
    pub(crate) children: [ArtNode; 4],
}

#[repr(C)]
pub(crate) struct ArtNode16 {
    pub(crate) base: ArtBaseNode,
    pub(crate) keys: [u8; 16],
    pub(crate) children: [ArtNode; 16],
}

#[repr(C)]
pub(crate) struct ArtNode48 {
    pub(crate) base: ArtBaseNode,
    pub(crate) keys: [u8; 256],
    pub(crate) children: [ArtNode; 48],
}

#[repr(C)]
pub(crate) struct ArtNode256 {
    base: ArtBaseNode,
    pub(crate) children: [ArtNode; 256],
}

pub(crate) struct ArtLeafNode {
    pub(crate) key: Vec<u8>,
    pub(crate) value: Vec<u8>,
}

impl ArtLeafNode {
    // <= PREFIX_SIZE
    pub(crate) fn key_prefix_match(&self, key_last_slice: &[u8], start: usize) -> usize {
        let mut i = 0;
        while i + start + 1 < self.key.len()
            && i + start + 1 < key_last_slice.len()
            && i < PREFIX_SIZE
        {
            if self.key[i + start] != key_last_slice[i + start] {
                break;
            }
            i += 1;
        }
        i
    }
}

#[repr(u8)]
pub(crate) enum ArtNodeType {
    Node4,
    Node16,
    Node48,
    Node256,
}

impl ArtBaseNode {
    fn new(prefix: &[u8], ty: ArtNodeType) -> Self {
        assert!(prefix.len() <= PREFIX_SIZE, "prefix too long");

        let mut prefix_buf = [0; PREFIX_SIZE];
        prefix_buf[..prefix.len()].copy_from_slice(prefix);

        Self {
            prefix: prefix_buf,
            prefix_len: prefix.len() as u8,
            ch_num: 0,
            ty,
        }
    }

    pub(crate) fn prefix_match(&self, key: &[u8]) -> usize {
        let mut i = 0;
        while i < self.prefix_len as usize && i < key.len() {
            if self.prefix[i] != key[i] {
                break;
            }
            i += 1;
        }
        i
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.prefix_len as usize
    }

    pub(crate) fn contains_prefix(&self) -> bool {
        self.prefix_len != 0
    }
}

impl ArtNode4 {
    pub(crate) fn new(prefix: &[u8]) -> Self {
        Self {
            base: ArtBaseNode::new(prefix, ArtNodeType::Node4),
            keys: [0; 4],
            children: [ArtNode::None; 4],
        }
    }

    pub(crate) fn insert(&mut self, key: u8, child: ArtNode) {
        todo!()
    }
}

impl ArtNode16 {
    pub(crate) fn new(prefix: &[u8]) -> Self {
        Self {
            base: ArtBaseNode::new(prefix, ArtNodeType::Node16),
            keys: [0; 16],
            children: [ArtNode::None; 16],
        }
    }
}

impl ArtNode48 {
    pub(crate) fn new(prefix: &[u8]) -> Self {
        Self {
            base: ArtBaseNode::new(prefix, ArtNodeType::Node48),
            keys: [0; 256],
            children: [ArtNode::None; 48],
        }
    }
}

impl ArtNode256 {
    pub(crate) fn new(prefix: &[u8]) -> Self {
        Self {
            base: ArtBaseNode::new(prefix, ArtNodeType::Node256),
            children: [ArtNode::None; 256],
        }
    }
}
