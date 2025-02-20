use std::{
    alloc::Layout,
    cell::RefCell,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::error::AllocError;

const ITEM_SIZE: usize = std::mem::size_of::<u64>();
const BLOCK_SIZE: usize = 4096 / ITEM_SIZE;
const NO_BLOCK_LIMIT: usize = BLOCK_SIZE / 4 * ITEM_SIZE;

struct BlockArenaInner {
    mems: Vec<Vec<u64>>,
    ptr: NonNull<u8>,
    remaining_size: usize,
    memory_usage: AtomicUsize,
}

impl BlockArenaInner {
    fn try_alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let tail = self.ptr.as_ptr();

        let (slop, aligned_ptr) = align_up(tail, layout.align());
        let need = slop + layout.size();
        if need > NO_BLOCK_LIMIT {
            // align from 8
            let ptr = self.try_alloc_new_block(layout.size())?;
            return Ok(ptr);
        }

        let (_tail, aligned_ptr, need) = if need > self.remaining_size {
            self.reload_block()?;
            let tail = self.ptr.as_ptr();
            let (slop, aligned_ptr) = align_up(tail, layout.align());
            let need = slop + layout.size();
            assert!(need <= self.remaining_size);
            (tail, aligned_ptr, need)
        } else {
            (tail, aligned_ptr, need)
        };

        let new_tail = aligned_ptr.wrapping_add(layout.size());
        unsafe {
            self.ptr = NonNull::new_unchecked(new_tail);
            self.remaining_size -= need;
            Ok(NonNull::new_unchecked(aligned_ptr))
        }
    }

    fn reload_block(&mut self) -> Result<(), AllocError> {
        let block = vec![0; BLOCK_SIZE];
        let ptr = block.as_ptr() as *mut u8;
        let cap = block.len() * ITEM_SIZE;

        self.mems.push(block);
        unsafe {
            self.ptr = NonNull::new_unchecked(ptr);
            self.remaining_size = cap;
        }

        self.memory_usage.fetch_add(cap, Ordering::SeqCst);

        Ok(())
    }

    fn try_alloc_new_block(&mut self, byte_size: usize) -> Result<NonNull<u8>, AllocError> {
        let size = (byte_size + ITEM_SIZE - 1) / ITEM_SIZE;

        let mem = vec![0; size];
        let ptr = mem.as_ptr() as *mut u8;
        let len = mem.len() * ITEM_SIZE;

        self.mems.push(mem);
        self.memory_usage.fetch_add(len, Ordering::SeqCst);

        unsafe { Ok(NonNull::new_unchecked(ptr)) }
    }

    fn memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::SeqCst)
    }
}

fn align_up(ptr: *mut u8, align: usize) -> (usize, *mut u8) {
    assert!(align.is_power_of_two());
    let slop = ptr.align_offset(align);
    (slop, ptr.wrapping_add(slop))
}

pub struct BlockArena {
    // inner: UnsafeCell<BlockArenaInner>, // RefCell ?
    inner: RefCell<BlockArenaInner>,
}

unsafe impl Send for BlockArena {}
unsafe impl Sync for BlockArena {}

impl BlockArena {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(BlockArenaInner {
                mems: Vec::new(),
                ptr: NonNull::dangling(),
                remaining_size: 0,
                memory_usage: AtomicUsize::new(0),
            }),
        }
    }

    pub fn alloc(&self, layout: Layout) -> NonNull<u8> {
        self.inner.borrow_mut().try_alloc(layout).unwrap()
    }

    pub fn memory_usage(&self) -> usize {
        self.inner.borrow().memory_usage()
    }
}
