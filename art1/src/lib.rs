use internal_node::InternalNode;
use leaf_node::LeafNode;
use node256::Node256;
use node_ptr::{InternalNodePtr, NodePtr};

pub(crate) mod internal_node;
pub(crate) mod leaf_node;
pub(crate) mod node16;
pub(crate) mod node256;
pub(crate) mod node4;
pub(crate) mod node48;
pub(crate) mod node_meta;
pub(crate) mod node_ptr;

pub struct Art {
    inner: ArtInner,
}

impl Art {
    pub fn new() -> Self {
        Self {
            inner: ArtInner::new(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.inner.get(key)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn insert(&mut self, key: &[u8], value: &[u8]) {
        self.inner.insert(key, value);
    }
}

pub(crate) struct ArtInner {
    root: InternalNodePtr, // node256,
    len: usize,
}

impl ArtInner {
    pub(crate) fn new() -> Self {
        Self {
            root: Node256::new_into_internal_node_ptr(&[]),
            len: 0,
        }
    }

    // not allow same key
    pub(crate) fn insert(&mut self, key: &[u8], value: &[u8]) {
        assert!(!key.is_empty(), "key must not be empty");

        let mut parent = self.root;
        let mut node = self.root.get(key[0]);
        let mut prefix_matched_len = 0;

        loop {
            // match node {
            //     Some(n) => {

            //     }

            //     None => {
            //         let leaf_ptr = LeafNode::new_into_node_ptr(key.to_vec(), value.to_vec());
            //         parent.insert_no_grow(key[prefix_matched_len], leaf_ptr);
            //         return;
            //     }
            // }

            let n = if let Some(n) = node {
                n
            } else {
                let leaf_ptr = LeafNode::new_into_node_ptr(key.to_vec(), value.to_vec());
                parent.insert_no_grow(key[prefix_matched_len], leaf_ptr);
                return;
            };

            match n {
                NodePtr::Internal(internal_node_ptr) => todo!(),
                NodePtr::Leaf(leaf) => {
                    // prefix matched
                }
            }
        }

        todo!()
    }

    pub(crate) fn get(&self, key: &[u8]) -> Option<&[u8]> {
        if key.is_empty() {
            return None;
        }

        let mut node = self.root.get(key[0]);
        let mut prefix_matched_len = 0;

        loop {
            let n = if let Some(n) = node { n } else { return None };
            match n {
                NodePtr::Internal(internal_node_ptr) => {
                    let (match_len, _cmp_res) =
                        internal_node_ptr.prefix_match(&key[prefix_matched_len..]);

                    if internal_node_ptr.prefix().len() != match_len {
                        return None;
                    }

                    node = internal_node_ptr.get(key[prefix_matched_len + match_len]);
                    prefix_matched_len += match_len + 1;
                }
                NodePtr::Leaf(leaf) => {
                    let leaf = unsafe { leaf.as_ref() };

                    if leaf.key == key {
                        return Some(&leaf.value);
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }
}
