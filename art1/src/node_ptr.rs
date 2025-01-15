use std::ptr::NonNull;

use crate::{
    internal_node::InternalNode, leaf_node::LeafNode, node16::Node16, node256::Node256,
    node4::Node4, node48::Node48,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum NodePtr {
    Internal(InternalNodePtr),
    Leaf(NonNull<LeafNode>),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum InternalNodePtr {
    Node4(NonNull<Node4>),
    Node16(NonNull<Node16>),
    Node48(NonNull<Node48>),
    Node256(NonNull<Node256>),
}

impl Into<NodePtr> for InternalNodePtr {
    fn into(self) -> NodePtr {
        NodePtr::Internal(self)
    }
}

// impl InternalNodePtr {
//     pub(crate) fn insert_with_grow<Pref, New>(
//         pref: NonNull<Pref>,
//         ref_pos: &mut Option<NonNull<NodePtr>>,
//         key: u8,
//         child: NodePtr,
//     ) {
//         todo!()
//     }
// }

impl InternalNode for InternalNodePtr {
    fn get(&self, key: u8) -> Option<NodePtr> {
        match self {
            InternalNodePtr::Node4(ptr) => unsafe { ptr.as_ref() }.get(key),
            InternalNodePtr::Node16(ptr) => unsafe { ptr.as_ref() }.get(key),
            InternalNodePtr::Node48(ptr) => unsafe { ptr.as_ref() }.get(key),
            InternalNodePtr::Node256(ptr) => unsafe { ptr.as_ref() }.get(key),
        }
    }

    fn prefix(&self) -> &crate::node_meta::Prefix {
        match self {
            InternalNodePtr::Node4(ptr) => unsafe { ptr.as_ref() }.prefix(),
            InternalNodePtr::Node16(ptr) => unsafe { ptr.as_ref() }.prefix(),
            InternalNodePtr::Node48(ptr) => unsafe { ptr.as_ref() }.prefix(),
            InternalNodePtr::Node256(ptr) => unsafe { ptr.as_ref() }.prefix(),
        }
    }

    fn children_len(&self) -> usize {
        match self {
            InternalNodePtr::Node4(ptr) => unsafe { ptr.as_ref() }.children_len(),
            InternalNodePtr::Node16(ptr) => unsafe { ptr.as_ref() }.children_len(),
            InternalNodePtr::Node48(ptr) => unsafe { ptr.as_ref() }.children_len(),
            InternalNodePtr::Node256(ptr) => unsafe { ptr.as_ref() }.children_len(),
        }
    }

    fn is_full(&self) -> bool {
        match self {
            InternalNodePtr::Node4(ptr) => unsafe { ptr.as_ref() }.is_full(),
            InternalNodePtr::Node16(ptr) => unsafe { ptr.as_ref() }.is_full(),
            InternalNodePtr::Node48(ptr) => unsafe { ptr.as_ref() }.is_full(),
            InternalNodePtr::Node256(ptr) => unsafe { ptr.as_ref() }.is_full(),
        }
    }

    fn new(_prefix: &[u8]) -> Self {
        unimplemented!()
    }

    fn new_into_internal_node_ptr(_prefix: &[u8]) -> InternalNodePtr
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn insert_no_grow(&mut self, key: u8, child: NodePtr) {
        match self {
            InternalNodePtr::Node4(ptr) => unsafe { ptr.as_mut() }.insert_no_grow(key, child),
            InternalNodePtr::Node16(ptr) => unsafe { ptr.as_mut() }.insert_no_grow(key, child),
            InternalNodePtr::Node48(ptr) => unsafe { ptr.as_mut() }.insert_no_grow(key, child),
            InternalNodePtr::Node256(ptr) => unsafe { ptr.as_mut() }.insert_no_grow(key, child),
        }
    }

    fn grow(&self) -> NodePtr {
        match self {
            InternalNodePtr::Node4(ptr) => unsafe { ptr.as_ref() }.grow(),
            InternalNodePtr::Node16(ptr) => unsafe { ptr.as_ref() }.grow(),
            InternalNodePtr::Node48(ptr) => unsafe { ptr.as_ref() }.grow(),
            InternalNodePtr::Node256(_) => unreachable!(),
        }
    }
}
