use std::ptr::NonNull;

use crate::{
    internal_node::InternalNode,
    node256::Node256,
    node_meta::InternalNodeMeta,
    node_ptr::{InternalNodePtr, NodePtr},
};

pub(crate) struct Node48 {
    pub(crate) meta: InternalNodeMeta,
    pub(crate) key: [u8; 256],
    pub(crate) children: [Option<NodePtr>; 48],
}

impl InternalNode for Node48 {
    fn get(&self, key: u8) -> Option<NodePtr> {
        let p = self.key[key as usize] as usize;
        if p < self.children_len() {
            self.children[p as usize]
        } else {
            None
        }
    }

    fn prefix(&self) -> &crate::node_meta::Prefix {
        self.meta.prefix()
    }

    fn children_len(&self) -> usize {
        self.meta.num_children as _
    }

    fn is_full(&self) -> bool {
        self.meta.num_children == 48
    }

    fn new(prefix: &[u8]) -> Self {
        Self {
            meta: InternalNodeMeta::with_prefix(prefix),
            key: [u8::MAX; 256],
            children: [None; 48],
        }
    }

    fn new_into_internal_node_ptr(prefix: &[u8]) -> InternalNodePtr
    where
        Self: Sized,
    {
        let ptr = Self::new_into_ptr(prefix);
        InternalNodePtr::Node48(ptr)
    }

    fn insert_no_grow(&mut self, key: u8, child: NodePtr) {
        assert!(!self.is_full());
        assert!(self.key[key as usize] == u8::MAX);

        self.key[key as usize] = self.children_len() as _;
        self.children[self.children_len()] = Some(child);

        self.meta.num_children += 1;
    }

    fn grow(&self) -> NodePtr {
        let mut new_node = Node256::new(self.prefix().as_slice());

        for i in 0..256 {
            if self.key[i] != u8::MAX {
                let child = self.children[self.key[i] as usize].unwrap();
                new_node.insert_no_grow(i as u8, child);
            }
        }

        NodePtr::Internal(InternalNodePtr::Node256(
            NonNull::new(Box::into_raw(Box::new(new_node))).unwrap(),
        ))
    }
}
