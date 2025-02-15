use std::ptr::NonNull;

pub(crate) struct NodeMeta {
    pub(crate) prefix: Vec<u8>,
    pub(crate) num_children: u16,
}

pub(crate) struct Node4 {
    meta: NodeMeta,
    keys: [u8; 4],
    children: [NodePtr; 4],
}

pub(crate) struct Node16 {
    meta: NodeMeta,
    keys: [u8; 16],
    children: [NodePtr; 16],
}

pub(crate) struct Node48 {
    meta: NodeMeta,
    keys: [u8; 256],
    children: [NodePtr; 48],
}

pub(crate) struct Node256 {
    meta: NodeMeta,
    children: [NodePtr; 256],
}

pub(crate) struct LeafNode {
    key: Vec<u8>,
    value: Vec<u8>,
}

impl LeafNode {
    pub(crate) fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { key, value }
    }

    pub(crate) fn from_key_value(key: &[u8], value: &[u8]) -> Self {
        Self::new(key.to_vec(), value.to_vec())
    }

    pub(crate) fn key(&self) -> &[u8] {
        &self.key
    }

    pub(crate) fn value(&self) -> &[u8] {
        &self.value
    }

    pub(crate) fn check_prefix(&self, key: &[u8], start: Option<usize>) -> usize {
        let start = start.unwrap_or(0);

        let mut l = start;
        let mut r = self.key.len().min(key.len());
        let mut ret = None;

        while l <= r {
            let mid = l.midpoint(r);
            if self.key[mid] == key[mid] {
                ret = Some(mid);
                l = mid + 1;
            } else {
                r = mid - 1;
            }
        }

        ret.unwrap_or(0)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum InternalNodePtr {
    Node4(NonNull<Node4>),
    Node16(NonNull<Node16>),
    Node48(NonNull<Node48>),
    Node256(NonNull<Node256>),
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct LeafNodePtr {
    ptr: NonNull<LeafNode>,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum NodePtr {
    Internal(InternalNodePtr),
    Leaf(LeafNodePtr),
    None,
}

impl NodeMeta {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        let mut i = 0;
        for (a, b) in self.prefix.iter().zip(key) {
            if a != b {
                break;
            }
            i += 1;
        }
        i
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.prefix.len()
    }
}

impl Node4 {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        self.meta.check_prefix(key)
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len()
    }
}

impl Node16 {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        self.meta.check_prefix(key)
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len()
    }
}

impl Node48 {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        self.meta.check_prefix(key)
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len()
    }
}

impl Node256 {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        self.meta.check_prefix(key)
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len()
    }
}

impl InternalNodePtr {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        unsafe {
            match self {
                InternalNodePtr::Node4(n) => n.as_ref().check_prefix(key),
                InternalNodePtr::Node16(n) => n.as_ref().check_prefix(key),
                InternalNodePtr::Node48(n) => n.as_ref().check_prefix(key),
                InternalNodePtr::Node256(n) => n.as_ref().check_prefix(key),
            }
        }
    }

    pub(crate) fn prefix_len(&self) -> usize {
        unsafe {
            match self {
                InternalNodePtr::Node4(n) => n.as_ref().prefix_len(),
                InternalNodePtr::Node16(n) => n.as_ref().prefix_len(),
                InternalNodePtr::Node48(n) => n.as_ref().prefix_len(),
                InternalNodePtr::Node256(n) => n.as_ref().prefix_len(),
            }
        }
    }

    pub(crate) fn get_child(&self, key: u8) -> Option<&NodePtr> {
        todo!()
    }

    pub(crate) fn get_child_mut(&mut self, key: u8) -> Option<&mut NodePtr> {
        todo!()
    }
}

impl LeafNodePtr {
    pub(crate) fn from_key_value(key: &[u8], value: &[u8]) -> Self {
        unsafe {
            Self {
                ptr: NonNull::new_unchecked(Box::into_raw(Box::new(LeafNode::from_key_value(
                    key, value,
                )))),
            }
        }
    }

    pub(crate) fn key(&self) -> &[u8] {
        self.as_ref().key()
    }

    pub(crate) fn value(&self) -> &[u8] {
        self.as_ref().value()
    }

    pub(crate) fn as_ref(&self) -> &LeafNode {
        unsafe { self.ptr.as_ref() }
    }

    pub(crate) fn as_mut(&mut self) -> &mut LeafNode {
        unsafe { self.ptr.as_mut() }
    }

    pub(crate) fn check_prefix(&self, key: &[u8], start: Option<usize>) -> usize {
        self.as_ref().check_prefix(key, start)
    }
}
