use std::{marker::PhantomData, ptr::NonNull, sync::atomic::AtomicU64};

use std::sync::atomic::Ordering::*;

#[derive(Debug, Clone, Copy)]
pub(crate) enum NodePtr {
    Internal { ptr: NonNull<InternalNode> },
    Leaf { ptr: NonNull<LeafNode> },
    None,
}

pub(crate) struct LeafNode {
    key: Vec<u8>,
    value: Vec<u8>,
}

impl LeafNode {
    pub(crate) fn key(&self) -> &[u8] {
        &self.key
    }

    pub(crate) fn value(&self) -> &[u8] {
        &self.value
    }

    pub(crate) fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { key, value }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum NodeType {
    Node4,
    Node16,
    Node48,
    Node256,
}

#[repr(C)]
pub(crate) struct InternalNode {
    // version: 62 bit | lock: 1 bit | obsolete: 1 bit |
    version: AtomicU64,

    node_type: NodeType,
    num_children: u16,
    prefix: [u8; Self::MAX_PREFIX_LEN],
    prefix_len: usize,
}

impl InternalNode {
    pub(crate) const MAX_PREFIX_LEN: usize = 10;

    pub(crate) fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.prefix_len
    }

    pub(crate) fn is_empty_prefix(&self) -> bool {
        self.prefix_len() == 0
    }

    pub(crate) fn prefix_matches(&self, key: &[u8]) -> usize {
        let mut i = 0;
        let mut len = self.prefix_len.min(Self::MAX_PREFIX_LEN).min(key.len());
        while i < len && self.prefix[i] == key[i] {
            i += 1;
        }
        i
    }
}

// Opt Lock

#[derive(Debug)]
pub enum LockError {
    VersionMismatch,
    Locked,
    Obsoleted,
}

use crossbeam::epoch::Guard;
use LockError::*;

fn mark_lock(version: u64) -> u64 {
    version + 0b10
}

// fn mark_obsolte(version: u64) -> u64 {
//     version | 0b01
// }

fn is_obsolted(version: u64) -> bool {
    version & 0b01 != 0
}

fn is_locked(version: u64) -> bool {
    version & 0b10 != 0
}

impl InternalNode {
    pub(crate) fn read<'a>(this: NonNull<Self>) -> Result<ReadGuard<'a>, LockError> {
        let version = Self::check_version(this)?;
        Ok(ReadGuard::new(this, version))
    }

    pub(crate) fn write<'a>(this: NonNull<Self>) -> Result<WriteGuard<'a>, LockError> {
        let version = Self::check_version(this)?;

        unsafe {
            match this.as_ref().version.compare_exchange(
                version,
                mark_lock(version),
                Acquire,
                Relaxed,
            ) {
                Ok(_) => Ok(WriteGuard::new(this)),
                Err(_) => Err(VersionMismatch),
            }
        }
    }

    pub(crate) fn check_version(this: NonNull<Self>) -> Result<u64, LockError> {
        let version = unsafe { this.as_ref().version.load(Acquire) };
        if is_locked(version) {
            return Err(Locked);
        }
        if is_obsolted(version) {
            return Err(Obsoleted);
        }
        Ok(version)
    }
}

pub(crate) struct ReadGuard<'a> {
    node: NonNull<InternalNode>,
    version: u64,
    _marker: PhantomData<&'a InternalNode>,
}

impl ReadGuard<'_> {
    fn new(node: NonNull<InternalNode>, version: u64) -> Self {
        Self {
            node,
            version,
            _marker: PhantomData,
        }
    }

    fn as_ref(&self) -> &InternalNode {
        unsafe { self.node.as_ref() }
    }

    pub(crate) fn check_version(&self) -> Result<(), LockError> {
        if self.version != InternalNode::check_version(self.node)? {
            return Err(VersionMismatch);
        } else {
            Ok(())
        }
    }

    pub(crate) fn unlock(self) -> Result<(), LockError> {
        self.check_version()?;
        Ok(())
    }

    pub(crate) fn upgrade<'a>(self) -> Result<WriteGuard<'a>, (Self, LockError)> {
        match InternalNode::write(self.node) {
            Ok(guard) => Ok(guard),
            Err(err) => return Err((self, err)),
        }
    }

    pub(crate) fn prefix_matches(&self, key: &[u8]) -> usize {
        self.as_ref().prefix_matches(key)
    }

    pub(crate) fn is_lazy_prefix(&self) -> bool {
        self.as_ref().prefix_len() > InternalNode::MAX_PREFIX_LEN
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.as_ref().prefix_len()
    }

    pub(crate) fn get_child(&self, key: u8) -> NodePtr {
        todo!()
    }

    pub(crate) fn is_full(&self) -> bool {
        todo!()
    }
}

pub(crate) struct WriteGuard<'a> {
    node: NonNull<InternalNode>,
    _marker: PhantomData<&'a mut InternalNode>,
}

impl WriteGuard<'_> {
    fn new(node: NonNull<InternalNode>) -> Self {
        Self {
            node,
            _marker: PhantomData,
        }
    }

    fn as_ref(&self) -> &InternalNode {
        unsafe { self.node.as_ref() }
    }

    fn as_mut(&mut self) -> &mut InternalNode {
        unsafe { self.node.as_mut() }
    }

    pub(crate) unsafe fn as_ptr(&mut self) -> NonNull<InternalNode> {
        self.node.clone()
    }

    pub(crate) fn mark_obsolte(&mut self) {
        self.as_ref().version.fetch_or(0b01, Release);
    }

    pub(crate) fn mark_obsolte_and_defer(mut self, guard: &Guard) {
        self.mark_obsolte();
        todo!();
    }

    pub(crate) fn insert_child(&mut self, key: u8, child: NodePtr) {
        todo!()
    }

    pub(crate) fn replace_child(&mut self, key: u8, child: NodePtr) -> NodePtr {
        todo!()
    }

    pub(crate) fn grow(mut self, _guard: &Guard) -> Result<Self, (Self, LockError)> {
        todo!();
    }
}

impl Drop for WriteGuard<'_> {
    fn drop(&mut self) {
        self.as_ref()
            .version
            .store(self.as_ref().version.load(Relaxed) + 1, Release);
    }
}

// Node

pub(crate) struct Node4 {
    base: InternalNode,
    keys: [u8; 4],
    children: [NodePtr; 4],
}

pub(crate) struct Node16 {
    base: InternalNode,
    keys: [u8; 16],
    children: [NodePtr; 16],
}

pub(crate) struct Node48 {
    base: InternalNode,
    keys: [u8; 256],
    children: [NodePtr; 48],
}

pub(crate) struct Node256 {
    base: InternalNode,
    children: [NodePtr; 256],
}
