use std::ptr::NonNull;

use crate::node_ptr::NodePtr;

#[derive(Debug)]
pub(crate) struct LeafNode {
    pub(crate) key: Vec<u8>,
    pub(crate) value: Vec<u8>,
}

impl LeafNode {
    pub(crate) fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { key, value }
    }

    pub(crate) fn new_into_ptr(key: Vec<u8>, value: Vec<u8>) -> NonNull<LeafNode> {
        let ptr = Box::into_raw(Box::new(Self::new(key, value)));
        NonNull::new(ptr).unwrap()
    }

    pub(crate) fn new_into_node_ptr(key: Vec<u8>, value: Vec<u8>) -> NodePtr {
        NodePtr::Leaf(LeafNode::new_into_ptr(key, value))
    }
}
