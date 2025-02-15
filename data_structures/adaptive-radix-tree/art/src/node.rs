use std::{mem::MaybeUninit, option, ptr::NonNull};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum NodeType {
    Node4,
    Node16,
    Node48,
    Node256,
}

const PARTIAL_SIZE: usize = 10;

#[repr(C)]
pub(crate) enum ArtNode {
    Intel(NonNull<ArtBaseNode>),
    Leaf(NonNull<ArtLeaf>),
}

#[repr(C)]
pub(crate) struct ArtBaseNode {
    pub(crate) partial: [u8; PARTIAL_SIZE],
    pub(crate) partial_len: usize,
    pub(crate) num_children: u16,
    pub(crate) node_type: NodeType,
}

#[repr(C)]
pub(crate) struct ArtNode4 {
    base: ArtBaseNode,
    keys: [u8; 4],
    children: [*mut ArtNode; 4],
}

#[repr(C)]
pub(crate) struct ArtNode16 {
    base: ArtBaseNode,
    keys: [u8; 16],
    children: [*mut ArtNode; 16],
}

#[repr(C)]
pub(crate) struct ArtNode48 {
    base: ArtBaseNode,
    keys: [u8; 256],
    children: [*mut ArtNode; 48],
}

#[repr(C)]
pub(crate) struct ArtNode256 {
    base: ArtBaseNode,
    children: [*mut ArtNode; 256],
}

pub(crate) struct ArtLeaf {
    pub(crate) key: Vec<u8>,
    pub(crate) value: Vec<u8>,
}

impl ArtBaseNode {
    pub(crate) fn as_n4(this: NonNull<ArtBaseNode>) -> NonNull<ArtNode4> {
        unsafe { NonNull::new_unchecked(this.cast().as_ptr() as *mut ArtNode4) }
    }

    pub(crate) fn as_n16(this: NonNull<ArtBaseNode>) -> NonNull<ArtNode16> {
        unsafe { NonNull::new_unchecked(this.cast().as_ptr() as *mut ArtNode16) }
    }

    pub(crate) fn as_n48(this: NonNull<ArtBaseNode>) -> NonNull<ArtNode48> {
        unsafe { NonNull::new_unchecked(this.cast().as_ptr() as *mut ArtNode48) }
    }

    pub(crate) fn as_n256(this: NonNull<ArtBaseNode>) -> NonNull<ArtNode256> {
        unsafe { NonNull::new_unchecked(this.cast().as_ptr() as *mut ArtNode256) }
    }
}

impl ArtNode4 {
    pub(crate) fn as_base(this: NonNull<ArtNode4>) -> NonNull<ArtBaseNode> {
        unsafe { NonNull::new_unchecked(this.cast().as_ptr() as *mut ArtBaseNode) }
    }

    pub(crate) unsafe fn get_child(&self, index: usize) -> NonNull<ArtNode> {
        let ptr = self.children[index];
        unsafe { NonNull::new_unchecked(ptr as *mut ArtNode) }
    }
}

impl ArtNode16 {
    pub(crate) fn as_base(this: NonNull<ArtNode16>) -> NonNull<ArtBaseNode> {
        unsafe { NonNull::new_unchecked(this.cast().as_ptr() as *mut ArtBaseNode) }
    }

    pub(crate) unsafe fn get_child(&self, index: usize) -> NonNull<ArtNode> {
        let ptr = self.children[index];
        unsafe { NonNull::new_unchecked(ptr as *mut ArtNode) }
    }
}

impl ArtNode48 {
    pub(crate) fn as_base(this: NonNull<ArtNode48>) -> NonNull<ArtBaseNode> {
        unsafe { NonNull::new_unchecked(this.cast().as_ptr() as *mut ArtBaseNode) }
    }

    pub(crate) unsafe fn get_child(&self, index: usize) -> NonNull<ArtNode> {
        let ptr = self.children[index];
        unsafe { NonNull::new_unchecked(ptr as *mut ArtNode) }
    }
}

impl ArtNode256 {
    pub(crate) fn as_base(this: NonNull<ArtNode256>) -> NonNull<ArtBaseNode> {
        unsafe { NonNull::new_unchecked(this.cast().as_ptr() as *mut ArtBaseNode) }
    }

    pub(crate) fn get_child(&self, index: usize) -> Option<NonNull<ArtNode>> {
        let ptr = self.children[index];
        if ptr.is_null() {
            return None;
        }

        unsafe { Some(NonNull::new_unchecked(ptr as *mut ArtNode)) }
    }
}

impl ArtLeaf {
    pub(crate) fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { key, value }
    }
}

impl ArtNode {
    pub(crate) fn drop_inner_node4(n: NonNull<ArtNode4>) {
        #[inline]
        fn to_drop(this: &ArtNode4) {
            for i in 0..this.base.num_children as usize {
                let ptr = unsafe { this.get_child(i) };
                ArtNode::drop_node(ptr);
            }
        }
        to_drop(unsafe { n.as_ref() });

        let _ = unsafe { Box::from_raw(n.as_ptr()) };
    }

    pub(crate) fn drop_inner_node16(n: NonNull<ArtNode16>) {
        #[inline]
        fn to_drop(this: &ArtNode16) {
            for i in 0..this.base.num_children as usize {
                let ptr = unsafe { this.get_child(i) };
                ArtNode::drop_node(ptr);
            }
        }
        to_drop(unsafe { n.as_ref() });

        let _ = unsafe { Box::from_raw(n.as_ptr()) };
    }

    pub(crate) fn drop_inner_node48(n: NonNull<ArtNode48>) {
        #[inline]
        fn to_drop(this: &ArtNode48) {
            for i in 0..this.base.num_children as usize {
                let ptr = unsafe { this.get_child(i) };
                ArtNode::drop_node(ptr);
            }
        }
        to_drop(unsafe { n.as_ref() });

        let _ = unsafe { Box::from_raw(n.as_ptr()) };
    }

    pub(crate) fn drop_inner_node256(n: NonNull<ArtNode256>) {
        #[inline]
        fn to_drop(this: &ArtNode256) {
            for i in 0..this.base.num_children as usize {
                let ptr = this.get_child(i);
                if let Some(ptr) = ptr {
                    ArtNode::drop_node(ptr);
                }
            }
        }
        to_drop(unsafe { n.as_ref() });

        let _ = unsafe { Box::from_raw(n.as_ptr()) };
    }

    #[inline]
    pub(crate) fn drop_inner_node(n: NonNull<ArtBaseNode>) {
        match unsafe { n.as_ref() }.node_type {
            NodeType::Node4 => Self::drop_inner_node4(n.cast()),
            NodeType::Node16 => Self::drop_inner_node16(n.cast()),
            NodeType::Node48 => Self::drop_inner_node48(n.cast()),
            NodeType::Node256 => Self::drop_inner_node256(n.cast()),
        }

        let _ = unsafe { Box::from_raw(n.as_ptr()) };
    }

    #[inline]
    pub(crate) fn drop_leaf_node(n: NonNull<ArtLeaf>) {
        let _ = unsafe { Box::from_raw(n.as_ptr()) };
    }

    #[inline]
    pub(crate) fn drop_node(n: NonNull<ArtNode>) {
        match unsafe { n.as_ref() } {
            ArtNode::Intel(n) => Self::drop_inner_node(n.clone()),
            ArtNode::Leaf(n) => Self::drop_leaf_node(n.clone()),
        }
    }
}

impl ArtBaseNode {
    pub(crate) fn get_child_order_ref_ptr(
        this: NonNull<ArtBaseNode>,
        byte: u8,
    ) -> Option<NonNull<*mut ArtNode>> {
        match unsafe { this.as_ref() }.node_type {
            NodeType::Node4 => {
                let this_ptr = this.cast::<ArtNode4>();
                let this = unsafe { this_ptr.as_ref() };
                for i in 0..this.base.num_children as usize {
                    if this.keys[i] == byte {
                        let ref_ch_ptr = NonNull::from(&this.children[i]);
                        return Some(ref_ch_ptr);
                    }
                }
            }
            NodeType::Node16 => {
                let this_ptr = this.cast::<ArtNode16>();
                let this = unsafe { this_ptr.as_ref() };
                for i in 0..this.base.num_children as usize {
                    if this.keys[i] == byte {
                        let ref_ch_ptr = NonNull::from(&this.children[i]);
                        return Some(ref_ch_ptr);
                    }
                }
            }
            NodeType::Node48 => {
                let this_ptr = this.cast::<ArtNode48>();
                let this = unsafe { this_ptr.as_ref() };
                let index = this.keys[byte as usize];
                if index == 0 {
                    return None;
                }

                let ref_ch_ptr = NonNull::from(&this.children[index as usize - 1]);
                return Some(ref_ch_ptr);
            }
            NodeType::Node256 => {
                let this_ptr = this.cast::<ArtNode256>();
                let this = unsafe { this_ptr.as_ref() };
                let index = this.children[byte as usize];
                if index.is_null() {
                    return None;
                }

                let ref_ch_ptr = NonNull::from(&this.children[byte as usize]);
                return Some(ref_ch_ptr);
            }
        }

        None
    }

    pub(crate) fn get_child_order_ptr(
        this: NonNull<ArtBaseNode>,
        byte: u8,
    ) -> Option<NonNull<ArtNode>> {
        let res = ArtBaseNode::get_child_order_ref_ptr(this, byte);

        match res {
            Some(ptr) => Some(unsafe { NonNull::new_unchecked(*ptr.as_ptr()) }),
            None => None,
        }
    }
}

impl ArtBaseNode {
    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        let mut i = 0;
        while i < self.partial_len && i < key.len() {
            if self.partial[i] != key[i] {
                return i;
            }
            i += 1;
        }
        i
    }
}
