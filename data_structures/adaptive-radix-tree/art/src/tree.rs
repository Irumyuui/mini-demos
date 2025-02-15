use std::ptr::NonNull;

use crate::node::{ArtBaseNode, ArtNode};

pub(crate) struct ArtInner {
    root: Option<NonNull<ArtNode>>,
    len: usize,
}

impl ArtInner {
    pub(crate) fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub(crate) fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let mut node = self.root;
        let mut matched_len = 0;

        while let Some(n) = node {
            match unsafe { n.as_ref() } {
                ArtNode::Intel(intel_ptr) => {
                    let intel = unsafe { intel_ptr.as_ref() };

                    if intel.partial_len != 0 {
                        let prefix_matched_len = intel.check_prefix(&key[matched_len..]);
                        if prefix_matched_len != intel.partial_len {
                            return None;
                        }
                        matched_len += prefix_matched_len;
                    }

                    let ch = ArtBaseNode::get_child_order_ptr(intel_ptr.clone(), key[matched_len]);
                    match ch {
                        Some(ch) => {
                            node = Some(ch);
                            matched_len += 1;
                        }
                        None => return None,
                    }
                }

                ArtNode::Leaf(leaf) => {
                    let leaf = unsafe { leaf.as_ref() };
                    if leaf.key == key {
                        return Some(&leaf.value);
                    }
                    return None;
                }
            }
        }

        None
    }

    pub fn insert(&mut self, key: &[u8], value: &[u8]) {
        
    }
}

impl Drop for ArtInner {
    fn drop(&mut self) {
        match self.root.take() {
            Some(ptr) => ArtNode::drop_node(ptr),
            None => return,
        }
    }
}
