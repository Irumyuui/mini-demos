use crate::{
    internal_node::InternalNode,
    node_meta::{InternalNodeMeta, Prefix},
    node_ptr::{InternalNodePtr, NodePtr},
};

pub(crate) struct Node256 {
    pub(crate) meta: InternalNodeMeta,
    children: [Option<NodePtr>; 256],
}

impl InternalNode for Node256 {
    fn get(&self, key: u8) -> Option<NodePtr> {
        self.children[key as usize]
    }

    fn prefix(&self) -> &Prefix {
        self.meta.prefix()
    }

    fn new(prefix: &[u8]) -> Self {
        Self {
            meta: InternalNodeMeta::with_prefix(prefix),
            children: [None; 256],
        }
    }

    fn new_into_internal_node_ptr(prefix: &[u8]) -> InternalNodePtr
    where
        Self: Sized,
    {
        let ptr = Self::new_into_ptr(prefix);
        InternalNodePtr::Node256(ptr)
    }

    fn children_len(&self) -> usize {
        self.meta.num_children as _
    }

    fn is_full(&self) -> bool {
        self.children_len() == 256
    }

    fn insert_no_grow(&mut self, key: u8, child: NodePtr) {
        assert!(!self.is_full());
        assert!(self.children[key as usize].is_none());

        self.children[key as usize] = Some(child);
        self.meta.num_children += 1;
    }

    fn grow(&self) -> NodePtr {
        unreachable!()
    }
}
