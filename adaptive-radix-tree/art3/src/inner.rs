use std::ptr::NonNull;

use crate::node::{LeafNode, LeafNodePtr, NodePtr};

pub(crate) struct ArtInner {
    root: NodePtr,
}

impl ArtInner {
    pub(crate) fn new() -> Self {
        Self {
            root: NodePtr::None,
        }
    }

    pub(crate) fn insert(&mut self, key: &[u8], value: &[u8]) {
        let cur = &mut self.root;
        let depth = 0;

        loop {
            match *cur {
                NodePtr::Internal(internal_node_ptr) => todo!(),

                NodePtr::Leaf(leaf) => {
                    // if leaf.key() == key {
                    //     panic!("key already exists");
                    // }

                    let prefix_len = leaf.check_prefix(&key[depth..], Some(depth));
                }

                NodePtr::None => {
                    *cur = NodePtr::Leaf(LeafNodePtr::from_key_value(key, value));
                    return;
                }
            }
        }
    }

    pub(crate) fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let mut cur = &self.root;
        let mut depth = 0;

        loop {
            match cur {
                NodePtr::None => return None,

                NodePtr::Internal(n) => {
                    if n.prefix_len() != n.check_prefix(&key[depth..]) {
                        return None;
                    }
                    depth += n.prefix_len();
                    match n.get_child(key[depth]) {
                        Some(child) => {
                            depth += 1;
                            cur = child;
                        }
                        None => return None,
                    }
                }

                NodePtr::Leaf(leaf) => {
                    if leaf.key() == key {
                        return Some(leaf.value());
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}
