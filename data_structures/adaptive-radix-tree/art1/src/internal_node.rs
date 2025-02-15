use std::ptr::NonNull;

use crate::{
    node_meta::Prefix,
    node_ptr::{InternalNodePtr, NodePtr},
};

pub(crate) trait InternalNode {
    fn insert_no_grow(&mut self, key: u8, child: NodePtr);

    fn grow(&self) -> NodePtr;

    fn get(&self, key: u8) -> Option<NodePtr>;

    fn prefix(&self) -> &Prefix;

    fn prefix_match(&self, prefix: &[u8]) -> (usize, std::cmp::Ordering) {
        let this = self.prefix().as_slice();
        let mut i = 0;

        while i < this.len() && i < prefix.len() {
            if this[i] != prefix[i] {
                return (i, this[i].cmp(&prefix[i]));
            }
            i += 1;
        }

        // (i, this.len().cmp(&prefix.len()))

        if this.len() <= prefix.len() {
            (i, std::cmp::Ordering::Equal)
        } else {
            (i, std::cmp::Ordering::Greater)
        }
    }

    fn children_len(&self) -> usize;

    fn is_full(&self) -> bool;

    fn new(prefix: &[u8]) -> Self;

    fn new_into_ptr(prefix: &[u8]) -> NonNull<Self>
    where
        Self: Sized,
    {
        let ptr = Box::into_raw(Box::new(Self::new(prefix)));
        let ptr = NonNull::new(ptr).unwrap();

        ptr
    }

    fn new_into_internal_node_ptr(prefix: &[u8]) -> InternalNodePtr
    where
        Self: Sized;

    fn new_into_node_ptr(prefix: &[u8]) -> NodePtr
    where
        Self: Sized,
    {
        let ptr = Self::new_into_internal_node_ptr(prefix);
        NodePtr::Internal(ptr)
    }
}
