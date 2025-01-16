use std::ptr::NonNull;

pub(crate) const PREFIX_SIZE: usize = 10;

#[derive(Clone, Copy)]
pub(crate) struct Prefix {
    prefix: [u8; PREFIX_SIZE],
    prefix_len: u8,
}

#[derive(Clone, Copy)]
pub(crate) struct NodeMeta {
    pub(crate) prefix: Prefix,
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

impl Prefix {
    pub(crate) fn new(slice: &[u8]) -> Self {
        assert!(slice.len() <= PREFIX_SIZE);
        let mut prefix = [0; PREFIX_SIZE];
        prefix[..slice.len()].copy_from_slice(slice);
        Self {
            prefix,
            prefix_len: slice.len() as u8,
        }
    }

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

    pub(crate) fn len(&self) -> usize {
        self.prefix_len as usize
    }
}

impl NodeMeta {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        self.prefix.check_prefix(key)
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

    pub(crate) fn new(prefix: Prefix) -> Self {
        Self {
            meta: NodeMeta {
                prefix,
                num_children: 0,
            },
            keys: [0; 4],
            children: [NodePtr::None; 4],
        }
    }

    pub(crate) fn grow(&self) -> Node16 {
        let mut new_node = Node16 {
            meta: self.meta,
            keys: [0; 16],
            children: [NodePtr::None; 16],
        };

        new_node.keys.copy_from_slice(&self.keys);
        new_node.children.copy_from_slice(&self.children);

        new_node
    }

    pub(crate) fn is_full(&self) -> bool {
        self.meta.num_children == 4
    }

    pub(crate) fn add_child(&mut self, key: u8, child: NodePtr) {
        assert!(!self.is_full());

        let mut i = 0;
        while i < self.meta.num_children as _ {
            if key < self.keys[i] {
                break;
            }
            i += 1;
        }

        for j in (i..self.meta.num_children as usize).rev() {
            self.keys[j + 1] = self.keys[j];
            self.children[j + 1] = self.children[j];
        }

        self.keys[i] = key;
        self.children[i] = child;
        self.meta.num_children += 1;
    }
}

impl Node16 {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        self.meta.check_prefix(key)
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len()
    }

    pub(crate) fn add_child(&mut self, key: u8, child: NodePtr) {
        assert!(!self.is_full());

        let mut i = 0;
        while i < self.meta.num_children as _ {
            if key < self.keys[i] {
                break;
            }
            i += 1;
        }

        for j in (i..self.meta.num_children as usize).rev() {
            self.keys[j + 1] = self.keys[j];
            self.children[j + 1] = self.children[j];
        }

        self.keys[i] = key;
        self.children[i] = child;
        self.meta.num_children += 1;
    }

    pub(crate) fn is_full(&self) -> bool {
        self.meta.num_children == 16
    }

    pub(crate) fn grow(&self) -> Node48 {
        let mut new_node = Node48 {
            meta: self.meta,
            keys: [u8::MAX; 256],
            children: [NodePtr::None; 48],
        };

        for i in 0..self.meta.num_children as usize {
            new_node.add_child(self.keys[i], self.children[i]);
        }

        new_node
    }
}

impl Node48 {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        self.meta.check_prefix(key)
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len()
    }

    pub(crate) fn add_child(&mut self, key: u8, child: NodePtr) {
        assert!(!self.is_full() && self.keys[key as usize] == u8::MAX);

        self.keys[key as usize] = self.meta.num_children as u8;
        self.children[self.meta.num_children as usize] = child;
        self.meta.num_children += 1;
    }

    pub(crate) fn is_full(&self) -> bool {
        self.meta.num_children == 48
    }

    pub(crate) fn grow(&self) -> Node256 {
        let mut new_node = Node256 {
            meta: self.meta,
            children: [NodePtr::None; 256],
        };

        for p in self.keys.iter() {
            if *p != u8::MAX {
                new_node.add_child(*p, self.children[*p as usize]);
            }
        }

        new_node
    }
}

impl Node256 {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        self.meta.check_prefix(key)
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.meta.num_children == 256
    }

    pub(crate) fn add_child(&mut self, key: u8, child: NodePtr) {
        assert!(!self.is_full() && self.children[key as usize] == NodePtr::None);
        self.children[key as usize] = child;
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

    pub(crate) fn is_full(&self) -> bool {
        unsafe {
            match self {
                InternalNodePtr::Node4(n) => n.as_ref().is_full(),
                InternalNodePtr::Node16(n) => n.as_ref().is_full(),
                InternalNodePtr::Node48(n) => n.as_ref().is_full(),
                InternalNodePtr::Node256(n) => n.as_ref().is_full(),
            }
        }
    }

    pub(crate) fn grow(&self) -> InternalNodePtr {
        unsafe {
            match self {
                InternalNodePtr::Node4(n) => InternalNodePtr::Node16(NonNull::new_unchecked(
                    Box::into_raw(Box::new(n.as_ref().grow())),
                )),
                InternalNodePtr::Node16(n) => InternalNodePtr::Node48(NonNull::new_unchecked(
                    Box::into_raw(Box::new(n.as_ref().grow())),
                )),
                InternalNodePtr::Node48(n) => InternalNodePtr::Node256(NonNull::new_unchecked(
                    Box::into_raw(Box::new(n.as_ref().grow())),
                )),
                InternalNodePtr::Node256(_) => unreachable!(),
            }
        }
    }

    // if grow
    pub(crate) fn add_child(
        this: &mut InternalNodePtr,
        key: u8,
        child: NodePtr,
    ) -> Option<InternalNodePtr> {
        fn add(n: &mut InternalNodePtr, key: u8, child: NodePtr) {
            unsafe {
                match n {
                    InternalNodePtr::Node4(n) => n.as_mut().add_child(key, child),
                    InternalNodePtr::Node16(n) => n.as_mut().add_child(key, child),
                    InternalNodePtr::Node48(n) => n.as_mut().add_child(key, child),
                    InternalNodePtr::Node256(n) => n.as_mut().add_child(key, child),
                }
            }
        }

        if this.is_full() {
            // let node = this.grow();
            // let pref = this.clone();
            // *this = node;

            // unsafe {
            //     // dealloc
            //     match pref {
            //         InternalNodePtr::Node16(n) => drop(Box::from_raw(n.as_ptr())),
            //         InternalNodePtr::Node4(n) => drop(Box::from_raw(n.as_ptr())),
            //         InternalNodePtr::Node48(n) => drop(Box::from_raw(n.as_ptr())),
            //         InternalNodePtr::Node256(n) => drop(Box::from_raw(n.as_ptr())),
            //     };
            // }

            let mut new_node = this.grow();
            add(&mut new_node, key, child);
            Some(new_node);
        }

        add(this, key, child);
        None
    }
}

impl LeafNodePtr {
    pub(crate) fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        unsafe {
            Self {
                ptr: NonNull::new_unchecked(Box::into_raw(Box::new(LeafNode::new(key, value)))),
            }
        }
    }

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
