use std::ptr::NonNull;

use crate::node::{
    ArtBaseNode, ArtLeafNode, ArtNode, ArtNode4, ArtNode16, ArtNode48, ArtNode256, ArtNodeType,
    PREFIX_SIZE,
};

pub(crate) struct ArtInner {
    root: NonNull<ArtNode256>,
}

impl ArtInner {
    pub(crate) fn new() -> Self {
        Self {
            root: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(ArtNode256::new(&[])))) },
        }
    }

    pub(crate) fn insert(&mut self, key: &[u8], value: &[u8]) {
        let node = unsafe { self.root.as_mut() }
            .children
            .get_mut(key[0] as usize)
            .unwrap() as *mut _;

        let mut pre_len = 0;

        'main: loop {
            match unsafe { &*node } {
                ArtNode::Internal(n) => {
                    
                },

                ArtNode::Leaf(n) => {
                    if unsafe { n.as_ref() }.key == key {
                        panic!("key already exists");
                    }

                    let old_leaf = n.clone();
                    let new_leaf = unsafe {
                        NonNull::new_unchecked(Box::into_raw(Box::new(ArtLeafNode {
                            key: key.to_vec(),
                            value: value.to_vec(),
                        })))
                    };

                    loop {
                        let prefix_len =
                            unsafe { old_leaf.as_ref() }.key_prefix_match(&key, pre_len);

                        let mut new_node4 = ArtNode4::new(&key[pre_len..prefix_len]);

                        if prefix_len < PREFIX_SIZE
                            || pre_len + prefix_len + 1 == key.len()
                            || pre_len + prefix_len + 1 == unsafe { old_leaf.as_ref().key.len() }
                        {
                            new_node4.insert(key[pre_len + prefix_len], ArtNode::Leaf(new_leaf));
                            new_node4.insert(
                                unsafe { old_leaf.as_ref() }.key[pre_len + prefix_len],
                                ArtNode::Leaf(old_leaf),
                            );
                            unsafe {
                                *node = ArtNode::Internal(
                                    NonNull::new_unchecked(Box::into_raw(Box::new(new_node4)))
                                        .cast(),
                                );
                            }

                            break 'main;
                        }
                        pre_len += prefix_len;

                        unsafe {
                            *node = ArtNode::Internal(
                                NonNull::new_unchecked(Box::into_raw(Box::new(new_node4))).cast(),
                            );
                        }
                    }
                }

                ArtNode::None => unsafe {
                    let new_leaf = ArtNode::Leaf(NonNull::new_unchecked(Box::into_raw(Box::new(
                        ArtLeafNode {
                            key: key.to_vec(),
                            value: value.to_vec(),
                        },
                    ))));
                    *node = new_leaf;
                    break 'main;
                },
            }
        }
    }

    pub(crate) fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let mut node = ArtNode::Internal(self.root.cast());
        let mut pre_len = 0;

        'main: loop {
            match &node {
                ArtNode::Internal(n) => {
                    if unsafe { n.as_ref() }.contains_prefix() {
                        let prefix_len = unsafe { n.as_ref() }.prefix_match(&key[pre_len..]);
                        if prefix_len != unsafe { n.as_ref() }.prefix_len() as usize {
                            return None;
                        }
                        pre_len += prefix_len;
                    }

                    let expected_key = key[pre_len];
                    match unsafe { n.as_ref() }.ty {
                        ArtNodeType::Node4 => {
                            let n: &ArtNode4 = unsafe { n.cast().as_ref() };
                            for i in 0..n.base.ch_num {
                                if n.keys[i as usize] == expected_key {
                                    node = n.children[i as usize];
                                    pre_len += 1;
                                    continue 'main;
                                }
                            }
                            break 'main;
                        }

                        ArtNodeType::Node16 => {
                            let n: &ArtNode16 = unsafe { n.cast().as_ref() };

                            for i in 0..n.base.ch_num {
                                if n.keys[i as usize] == expected_key {
                                    node = n.children[i as usize];
                                    pre_len += 1;
                                    continue 'main;
                                }
                            }

                            break 'main;
                        }

                        ArtNodeType::Node48 => {
                            let n: &ArtNode48 = unsafe { n.cast().as_ref() };
                            let pos = n.keys[expected_key as usize];

                            if pos == u8::MAX {
                                break 'main;
                            }

                            pre_len += 1;
                            node = n.children[pos as usize];
                        }

                        ArtNodeType::Node256 => {
                            let n: &ArtNode256 = unsafe { n.cast().as_ref() };

                            pre_len += 1;
                            node = n.children[expected_key as usize];
                        }
                    }
                }

                ArtNode::Leaf(n) => {
                    let n = unsafe { n.as_ref() };
                    match n.key == key {
                        true => return Some(&n.value),
                        false => return None,
                    }
                }

                ArtNode::None => return None,
            }
        }

        None
    }
}

impl Drop for ArtInner {
    fn drop(&mut self) {
        drop_n256(self.root);
    }
}

// Drop

fn drop_tree(node: ArtNode) {
    match node {
        ArtNode::Internal(n) => drop_internal(n),
        ArtNode::Leaf(n) => drop_leaf(n),
        ArtNode::None => return,
    }
}

#[inline]
fn drop_internal(node: NonNull<ArtBaseNode>) {
    match unsafe { node.as_ref() }.ty {
        ArtNodeType::Node4 => drop_n4(node.cast()),
        ArtNodeType::Node16 => drop_n16(node.cast()),
        ArtNodeType::Node48 => drop_n48(node.cast()),
        ArtNodeType::Node256 => drop_n256(node.cast()),
    }
}

#[inline]
fn drop_n4(node: NonNull<ArtNode4>) {
    let n = unsafe { node.as_ref() };
    for i in 0..n.base.ch_num {
        match n.children[i as usize] {
            ArtNode::Internal(n) => drop_internal(n),
            ArtNode::Leaf(n) => drop_leaf(n),
            ArtNode::None => unreachable!(),
        }
    }

    unsafe {
        drop(Box::from_raw(node.as_ptr()));
    }
}

#[inline]
fn drop_n16(node: NonNull<ArtNode16>) {
    let n = unsafe { node.as_ref() };
    for i in 0..n.base.ch_num {
        match n.children[i as usize] {
            ArtNode::Internal(n) => drop_internal(n),
            ArtNode::Leaf(n) => drop_leaf(n),
            ArtNode::None => unreachable!(),
        }
    }
    unsafe {
        drop(Box::from_raw(node.as_ptr()));
    }
}

#[inline]
fn drop_n48(node: NonNull<ArtNode48>) {
    let n = unsafe { node.as_ref() };
    for i in 0..n.base.ch_num {
        match n.children[i as usize] {
            ArtNode::Internal(n) => drop_internal(n),
            ArtNode::Leaf(n) => drop_leaf(n),
            ArtNode::None => unreachable!(),
        }
    }
    unsafe {
        drop(Box::from_raw(node.as_ptr()));
    }
}

#[inline]
fn drop_n256(node: NonNull<ArtNode256>) {
    for ch in unsafe { node.as_ref() }.children.into_iter() {
        match ch {
            ArtNode::Internal(n) => drop_internal(n),
            ArtNode::Leaf(n) => drop_leaf(n),
            ArtNode::None => continue,
        }
    }

    unsafe {
        drop(Box::from_raw(node.as_ptr()));
    }
}

#[inline]
fn drop_leaf(node: NonNull<ArtLeafNode>) {
    unsafe {
        drop(Box::from_raw(node.as_ptr()));
    }
}
