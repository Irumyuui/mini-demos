use std::ptr::NonNull;

use crate::{
    internal_node::InternalNode,
    node48::Node48,
    node_meta::InternalNodeMeta,
    node_ptr::{InternalNodePtr, NodePtr},
};

pub(crate) struct Node16 {
    pub(crate) meta: InternalNodeMeta,
    pub(crate) key: [u8; 16],
    pub(crate) children: [Option<NodePtr>; 16],
}

impl InternalNode for Node16 {
    fn get(&self, key: u8) -> Option<NodePtr> {
        for (i, e) in self.key.iter().take(self.children_len()).enumerate() {
            if *e == key {
                return self.children[i];
            }
        }

        None
    }

    fn prefix(&self) -> &crate::node_meta::Prefix {
        self.meta.prefix()
    }

    fn new(prefix: &[u8]) -> Self {
        Self {
            meta: InternalNodeMeta::with_prefix(prefix),
            key: [u8::MAX; 16],
            children: [None; 16],
        }
    }

    fn new_into_internal_node_ptr(prefix: &[u8]) -> InternalNodePtr
    where
        Self: Sized,
    {
        let ptr = Self::new_into_ptr(prefix);
        InternalNodePtr::Node16(ptr)
    }

    fn children_len(&self) -> usize {
        self.meta.num_children as _
    }

    fn is_full(&self) -> bool {
        self.meta.num_children == 16
    }

    fn insert_no_grow(&mut self, key: u8, child: NodePtr) {
        assert!(!self.is_full());

        let mut i = 0;
        while i < self.children_len() {
            if self.key[i] > key {
                break;
            }
            i += 1;
        }

        for j in (i..self.children_len()).rev() {
            self.key[j + 1] = self.key[j];
            self.children[j + 1] = self.children[j];
        }

        self.key[i] = key;
        self.children[i] = Some(child);

        self.meta.num_children += 1;
    }

    fn grow(&self) -> NodePtr {
        let mut new_node = Node48::new(self.prefix().as_slice());
        for (i, e) in self.key.iter().enumerate().take(self.children_len()) {
            new_node.key[*e as usize] = i as u8;
            new_node.children[i] = self.children[i];
        }
        new_node.meta.num_children = self.children_len() as _;
        NodePtr::Internal(InternalNodePtr::Node48(
            NonNull::new(Box::into_raw(Box::new(new_node))).unwrap(),
        ))
    }
}
