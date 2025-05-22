use std::{cell::RefCell, os::fd::AsRawFd, rc::Rc};

use rustix::io;
use rustix_uring::{IoUring, cqueue};
use tracing::instrument;

use crate::utils::slab::Slab;

use super::op::Lifecycle;

pub(crate) struct Driver {
    pub(crate) ops: Ops,
    pub(crate) uring: IoUring,
}

impl Driver {
    pub(crate) fn new(builder: &rustix_uring::Builder, entries: u32) -> std::io::Result<Self> {
        Ok(Self {
            uring: builder.build(entries)?,
            ops: Ops::new(),
        })
    }

    /// Once submit, will submit sq and check cq.
    /// If some completed, call to complete.
    pub(crate) fn submit(&mut self) -> std::io::Result<()> {
        loop {
            match self.uring.submit() {
                Ok(_) => {
                    self.uring.submission().sync();
                    return Ok(());
                }
                Err(e) => match e {
                    io::Errno::BUSY | io::Errno::AGAIN => self.tick(),
                    io::Errno::INTR => return Err(std::io::Error::from(e)),
                    _ => continue,
                },
            }
        }
    }

    pub(crate) fn tick(&mut self) {
        let mut cq = self.uring.completion();
        cq.sync();

        for cqe in cq {
            if cqe.user_data_u64() == u64::MAX {
                // Mark ignored IO.
                continue;
            }

            let index = cqe.user_data_u64();
            let result = match cqe.result() {
                n if n < 0 => Err(std::io::Error::from_raw_os_error(-n)),
                n => Ok(n as u32),
            };
            let flags = cqe.flags();

            self.ops.complete(index as usize, result, flags);
        }
    }

    fn num_op(&self) -> usize {
        self.ops.lifecycle.len()
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        while self.num_op() > 0 {
            let _ = self.uring.submit_and_wait(1);
            self.tick();
        }
    }
}

pub(crate) struct Ops {
    pub(crate) lifecycle: Slab<Lifecycle>,
}

impl Ops {
    fn new() -> Self {
        Self {
            lifecycle: Slab::with_capacity(256),
        }
    }

    pub(crate) fn complete(
        &mut self,
        index: usize,
        result: std::io::Result<u32>,
        flags: cqueue::Flags,
    ) {
        if self.lifecycle[index].complete(result, flags) {
            self.lifecycle.remove(index);
        }
    }
}

pub(crate) struct Context {
    inner: RefCell<Option<Rc<RefCell<Driver>>>>,
}

impl Context {
    pub const fn new() -> Self {
        Self {
            inner: RefCell::new(None),
        }
    }

    pub fn set(&self, driver: Rc<RefCell<Driver>>) {
        let res = self.inner.borrow_mut().replace(driver);
        assert!(res.is_none(), "Driver context already set");
    }

    pub fn unset(&self) {
        let res = self.inner.borrow_mut().take();
        assert!(res.is_some(), "Driver context not set");
    }

    pub fn handle(&self) -> Option<Rc<RefCell<Driver>>> {
        self.inner.borrow().as_ref().cloned()
    }
}

thread_local! {
    pub(crate) static CONTEXT: Context = const { Context::new() };
}

#[derive(Clone)]
pub(crate) struct Handle {
    pub(crate) inner: Rc<RefCell<Driver>>,
}

impl AsRawFd for Handle {
    fn as_raw_fd(&self) -> i32 {
        self.inner.borrow().uring.as_raw_fd()
    }
}
