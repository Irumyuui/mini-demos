use std::{mem::MaybeUninit, ptr::NonNull};

use crossbeam::epoch::Guard;

use crate::node::{InternalNode, LeafNode, LockError, Node256, NodePtr, ReadGuard};

pub struct Art {}

pub struct ArtInner {
    root: NonNull<Node256>,
}

impl ArtInner {
    fn get_inner<'a>(&'a self, key: &[u8], _guard: &Guard) -> Result<Option<&'a [u8]>, LockError> {
        let mut depth = 0;
        let mut cur = InternalNode::read(self.root.cast())?;

        loop {
            let prefix_len = cur.prefix_matches(&key[depth..]);
            if prefix_len != cur.prefix_len() {
                cur.unlock()?;
                return Ok(None);
            }

            let next_child = cur.get_child(key[depth + prefix_len]);
            cur.check_version()?;

            match next_child {
                NodePtr::Internal { ptr } => {
                    let mut guard = InternalNode::read(ptr.clone())?;
                    depth += 1;
                    std::mem::swap(&mut cur, &mut guard);
                    guard.unlock()?;
                }

                NodePtr::Leaf { ptr } => {
                    let leaf = unsafe { ptr.as_ref() };

                    let res = if leaf.key() == key {
                        Some(leaf.value())
                    } else {
                        None
                    };

                    cur.unlock()?;
                    return Ok(res);
                }

                NodePtr::None => {
                    cur.unlock()?;
                    return Ok(None);
                }
            }
        }
    }

    fn get<'a>(&'a self, key: &[u8], _guard: &Guard) -> Option<&'a [u8]> {
        'retry: loop {
            match self.get_inner(key, _guard) {
                Ok(res) => return res,
                Err(_) => continue 'retry,
            }
        }
    }

    // fn insert(&self, key: Vec<u8>, value: Vec<u8>, guard: &Guard) {
    //     'retry: loop {
    //         let cur = match InternalNode::read(self.root.cast()) {
    //             Ok(cur) => cur,
    //             Err(_) => continue 'retry,
    //         };
    //         let mut depth = 0;

    //         'inner: loop {
    //             let prefix_len = cur.prefix_matches(&key[depth..]);
    //             if cur.check_version().is_err() {
    //                 continue 'retry;
    //             }

    //             if prefix_len != cur.prefix_len() {
    //                 return;
    //             }

    //             depth += prefix_len;
    //             let next_node = cur.get_child(key[depth + prefix_len]);
    //             if cur.check_version().is_err() {
    //                 continue 'retry;
    //             }

    //             match next_node {
    //                 NodePtr::Internal { ptr } => todo!(),

    //                 NodePtr::Leaf { ptr } => todo!(),

    //                 NodePtr::None => {
    //                     let guard = match cur.upgrade() {
    //                         Ok(g) => g,
    //                         Err(_) => continue 'retry,
    //                     };

    //                     let leaf_pos = key[depth + prefix_len];

    //                 }
    //             }

    //             todo!()
    //         }
    //     }
    // }

    fn insert_inner(&self, key: &[u8], value: &[u8], guard: &Guard) -> Result<(), LockError> {
        let mut depth = 0;

        let mut parent: Option<ReadGuard<'_>> = None;
        let cur = InternalNode::read(self.root.cast())?;

        loop {
            let prefix_len = cur.prefix_matches(&key[depth..]);
            cur.check_version()?;

            if prefix_len != cur.prefix_len() {
                todo!()
            }

            depth += prefix_len;

            depth += prefix_len;
            let child = cur.get_child(key[depth]);
            cur.check_version()?;

            match child {
                NodePtr::Internal { ptr } => todo!(),

                NodePtr::Leaf { ptr } => todo!(),

                NodePtr::None => match cur.is_full() {
                    true => {
                        let _parent_lock = parent.unwrap().upgrade().map_err(|e| e.1)?;
                        let mut cur_lock = cur.upgrade().map_err(|e| e.1)?;

                        // insert a grow node.

                        cur_lock.mark_obsolte_and_defer(guard);

                        return Ok(());
                    }

                    false => {
                        let mut cur_lock = cur.upgrade().map_err(|e| e.1)?;
                        if let Some(parent) = parent.take() {
                            parent.unlock()?;
                        }

                        let new_leaf_pos = key[depth];

                        let new_leaf = NodePtr::Leaf {
                            ptr: NonNull::new(Box::into_raw(Box::new(LeafNode::new(
                                key.to_vec(),
                                value.to_vec(),
                            ))))
                            .unwrap(),
                        };

                        cur_lock.insert_child(new_leaf_pos, new_leaf);
                        return Ok(());
                    }
                },
            }
        }
    }

    fn insert(&self, key: &[u8], value: &[u8], guard: &Guard) {
        'retry: loop {
            match self.insert_inner(key, value, guard) {
                Ok(_) => return,
                Err(_) => continue 'retry,
            }
        }
    }
}
