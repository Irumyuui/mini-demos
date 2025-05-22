use std::os::fd::RawFd;

use rustix_uring::types;

use crate::uring::{driver::CONTEXT, op::Op};

pub(crate) struct Close;

impl Op<Close> {
    pub(crate) fn close(fd: RawFd) -> std::io::Result<Self> {
        CONTEXT.with(|c| {
            let c = c.handle().expect("no context");
            Op::submit_with(Close, |_| {
                rustix_uring::opcode::Close::new(types::Fd(fd)).build()
            })
        })
    }
}
