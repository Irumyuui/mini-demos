use std::{collections::BTreeMap, marker::PhantomData, ptr::NonNull};

use crate::node::{InternalNode, NodePtr, NodeType};

pub struct Art {
    inner: RawArt,
}

impl Art {
    pub fn new() -> Self {
        Self {
            inner: RawArt {
                root: NodePtr::None,
                _marker: PhantomData,
            },
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.inner.insert(key, value);
    }
}

struct RawArt {
    root: NodePtr,
    _marker: PhantomData<BTreeMap<Vec<u8>, Vec<u8>>>,
}

unsafe impl Send for RawArt {}

impl RawArt {
    pub(crate) fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let mut node = &self.root;
        let mut depth = 0;

        loop {
            match node {
                NodePtr::Internal(n) => unsafe {
                    let prefix_len = n.as_ref().check_prefix(&key[depth..]);
                    if n.as_ref().prefix_len() != prefix_len {
                        return None;
                    }
                    depth += prefix_len;

                    let child = n.as_ref().get_child(key[depth]);
                    match child {
                        Some(n) => {
                            node = n;
                            depth += 1;
                        }
                        None => return None,
                    }
                },

                NodePtr::Leaf(n) => unsafe {
                    let leaf = n.as_ref();

                    if leaf.key.as_slice() == key {
                        return Some(leaf.value.as_slice());
                    } else {
                        return None;
                    }
                },

                NodePtr::None => return None,
            }
        }
    }

    pub(crate) fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let mut cur = NonNull::new(&mut self.root as _).unwrap();
        let mut ref_pos = &mut self.root;
        let mut depth = 0;

        'main: loop {
            unsafe {
                match cur.as_mut() {
                    NodePtr::Internal(raw_ptr) => {
                        if raw_ptr.as_ref().prefix_len() != 0 {
                            let prefix_diff =
                                InternalNode::prefix_mismatch(raw_ptr.clone(), &key, depth);

                            if prefix_diff < raw_ptr.as_ref().prefix_len() {
                                let mut new_node = InternalNode::new_lazy_prefix(
                                    &raw_ptr.as_ref().prefix()
                                        [prefix_diff.min(InternalNode::MAX_PREFIX_SIZE)..],
                                    prefix_diff,
                                    NodeType::Node4,
                                );

                                if raw_ptr.as_ref().prefix_len() <= InternalNode::MAX_PREFIX_SIZE {
                                    new_node.as_mut().add_child(
                                        raw_ptr.as_ref().prefix()[prefix_diff],
                                        NodePtr::Internal(raw_ptr.clone()),
                                    );

                                    let raw_ref = raw_ptr.as_mut();
                                    let mut i = 0;
                                    let mut j = prefix_diff + 1;
                                    while j < raw_ref.prefix_len {
                                        raw_ref.prefix[i] = raw_ref.prefix[j];
                                        i += 1;
                                        j += 1;
                                    }
                                    raw_ref.prefix_len -= prefix_diff + 1;
                                } else {
                                    let leaf =
                                        NodePtr::find_min_leaf(&NodePtr::Internal(raw_ptr.clone()))
                                            .expect("Internal node is empty");
                                    let pos = leaf.as_ref().key[depth + prefix_diff];
                                    new_node
                                        .as_mut()
                                        .add_child(pos, NodePtr::Internal(raw_ptr.clone()));

                                    let raw_ref = raw_ptr.as_mut();
                                    raw_ref.prefix_len -= prefix_diff + 1;

                                    let mut i = 0;
                                    let r = raw_ref.prefix_len.min(InternalNode::MAX_PREFIX_SIZE);
                                    while i < r {
                                        raw_ref.prefix[i] =
                                            leaf.as_ref().key[i + prefix_diff + depth + 1];
                                        i += 1;
                                    }
                                }

                                let pos = key[depth + prefix_diff];
                                let new_leaf = NodePtr::new_leaf(key, value);
                                new_node.as_mut().add_child(pos, new_leaf);

                                *ref_pos = NodePtr::Internal(new_node);
                                return;
                            }
                            depth += prefix_diff;
                        }

                        let child = raw_ptr.as_mut().get_child_mut(key[depth]);
                        match child {
                            Some(ch) => {
                                cur = NonNull::new(ch as *mut _).unwrap();
                                ref_pos = ch;
                                depth += 1;
                                continue 'main;
                            }
                            None => {
                                let mut node = if raw_ptr.as_ref().is_full() {
                                    let new_node = raw_ptr.as_mut().grow();
                                    *ref_pos = NodePtr::Internal(new_node);
                                    std::mem::drop(Box::from_raw(raw_ptr.as_ptr()));
                                    new_node
                                } else {
                                    raw_ptr.clone()
                                };

                                let byte = key[depth];
                                let child = NodePtr::new_leaf(key, value);
                                node.as_mut().add_child(byte, child);

                                return;
                            }
                        }
                    }

                    NodePtr::Leaf(old_leaf_raw_ptr) => {
                        if old_leaf_raw_ptr.as_ref().key == key {
                            panic!("key already exists");
                        }

                        let longest_prefix =
                            old_leaf_raw_ptr.as_ref().longest_common_prefix(&key, depth);

                        let mut new_node = InternalNode::new_lazy_prefix(
                            &key[depth
                                ..(depth + longest_prefix.min(InternalNode::MAX_PREFIX_SIZE))],
                            longest_prefix,
                            NodeType::Node4,
                        );

                        let old_byte = old_leaf_raw_ptr.as_ref().key[depth + longest_prefix];
                        let old_leaf_ptr = NodePtr::Leaf(old_leaf_raw_ptr.clone());
                        new_node.as_mut().add_child(old_byte, old_leaf_ptr);

                        let new_byte = key[depth + longest_prefix];
                        let new_leaf_ptr = NodePtr::new_leaf(key, value);
                        new_node.as_mut().add_child(new_byte, new_leaf_ptr);

                        *ref_pos = NodePtr::Internal(new_node);
                        return;
                    }

                    NodePtr::None => {
                        let new_leaf = NodePtr::new_leaf(key, value);
                        *ref_pos = new_leaf;
                        return;
                    }
                }
            }
        }
    }
}

// impl Drop for RawArt {
//     fn drop(&mut self) {

//     }
// }
