use std::{num::IntErrorKind, ptr::NonNull};

use crate::node::{InternalNodePtr, LeafNodePtr, Node4, NodePtr, PREFIX_SIZE, Prefix};

pub(crate) struct ArtInner {
    root: NodePtr,
}

impl ArtInner {
    pub(crate) fn new() -> Self {
        Self {
            root: NodePtr::None,
        }
    }

    pub(crate) fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let mut node: &NodePtr = &self.root;
        let mut depth = 0;

        loop {
            match node {
                NodePtr::Internal(internal_node_ptr) => {
                    let prefix_len = internal_node_ptr.check_prefix(&key[depth..]);

                    if prefix_len != internal_node_ptr.prefix_len() {
                        return None;
                    }
                    depth += prefix_len;

                    let child = internal_node_ptr.get_child(key[depth]);
                    match child {
                        Some(node_ptr) => {
                            node = node_ptr;
                            depth += 1;
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

                NodePtr::None => return None,
            }
        }
    }

    pub(crate) fn insert_inner(mut node: &mut NodePtr, key: Vec<u8>, value: Vec<u8>) {
        let mut depth = 0;

        'main: loop {
            match node {
                NodePtr::Internal(internal_node_ptr) => {
                    if internal_node_ptr.prefix_len() != 0 {
                        let prefix_diff = internal_node_ptr.check_prefix(&key[depth..]);

                        if prefix_diff < internal_node_ptr.prefix_len() {
                            return;
                        }

                        depth += internal_node_ptr.prefix_len();
                    }

                    let child = internal_node_ptr.get_child_mut(key[depth]);

                    match child {
                        Some(ch) => {
                            depth += 1;
                            node = ch;
                            continue 'main;
                        }
                        None => {
                            let k = key[depth];
                            let new_leaf = NodePtr::Leaf(LeafNodePtr::new(key, value));
                            // internal_node_ptr.add_child(k, new_leaf);

                            // InternalNodePtr::add_child(internal_node_ptr, k, new_leaf);

                            

                            return;

                            // unsafe {
                            //     let ptr = internal_node_ptr as *mut InternalNodePtr;
                            //     (*ptr).add_child(k, new_leaf);
                            // }
                        }
                    }

                    // if let Some(ch) = child {
                    //     depth += 1;
                    //     node = ch;
                    //     continue 'main;
                    // }

                    // let k = key[depth];
                    // let new_leaf = NodePtr::Leaf(LeafNodePtr::new(key, value));
                    // // internal_node_ptr.add_child(k, new_leaf);
                    // InternalNodePtr::add_child(internal_node_ptr, k, new_leaf);

                    // return;
                }

                NodePtr::Leaf(old_leaf) => {
                    if old_leaf.key() == key {
                        panic!("same key");
                    }

                    let longest_prefix = old_leaf.check_prefix(&key, Some(depth));
                    let prefix =
                        Prefix::new(&key[depth..(depth + longest_prefix).min(PREFIX_SIZE)]);

                    let mut new_node = Node4::new(prefix);

                    new_node.add_child(
                        old_leaf.key()[depth + longest_prefix],
                        NodePtr::Leaf(old_leaf.clone()),
                    );
                    new_node.add_child(
                        key[depth + longest_prefix],
                        NodePtr::Leaf(LeafNodePtr::new(key, value)),
                    );

                    let new_node = NodePtr::Internal(InternalNodePtr::Node4(
                        NonNull::new(Box::into_raw(Box::new(new_node))).unwrap(),
                    ));
                    *node = new_node;
                    return;
                }

                NodePtr::None => {
                    *node = NodePtr::Leaf(LeafNodePtr::new(key, value));
                    return;
                }
            }
        }
    }

    pub(crate) fn insert(&mut self, mut key: Vec<u8>, value: Vec<u8>) {
        // let mut node: &mut NodePtr = &mut self.root;

        // loop {
        //     match node {
        //         NodePtr::Internal(internal_node_ptr) => todo!(),
        //         NodePtr::Leaf(leaf_node_ptr) => {

        //         },

        //         NodePtr::None => {
        //             *node = NodePtr::Leaf(LeafNodePtr::new(key, value));
        //             return;
        //         }
        //     }
        // }

        match key.last() {
            Some(x) if *x != 0 => key.push(0),
            Some(_) => {}
            None => panic!("Key is empty"),
        }

        // Self::insert_inner(&mut self.root, &mut &mut self.root, key, value);

        todo!()
    }
}
