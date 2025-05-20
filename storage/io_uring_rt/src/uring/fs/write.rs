use std::{
    future::poll_fn,
    pin::Pin,
    task::{Poll, ready},
};

use rustix_uring::{opcode, types};

use crate::uring::{
    op::{CompleteAble, Op},
    prelude::BufResult,
};

use super::{AsIoVec, shared_fd::SharedFd};

pub struct Write<T> {
    fd: SharedFd,
    buf: T,
}

impl<T> Write<T> {
    fn new(fd: SharedFd, buf: T) -> Self {
        Self { fd, buf }
    }
}

impl<T> Op<Write<T>>
where
    T: AsIoVec,
{
    pub fn write_at(fd: &SharedFd, buf: T, offset: u64) -> std::io::Result<Self> {
        Op::submit_with(Write::new(fd.clone(), buf), |write| {
            let fd = write.fd.raw_fd();
            let (ptr, len) = write.buf.as_io_vec();

            opcode::Write::new(types::Fd(fd), ptr, len as _)
                .offset(offset)
                .build()
        })
    }
}

impl<T> CompleteAble for Write<T>
where
    T: AsIoVec,
{
    type Output = BufResult<T>;

    fn handle_completion(comp: crate::uring::op::Completion<Self>) -> Self::Output {
        let res = comp.result.map(|res| res as usize);
        let buf = comp.data.buf;
        (res, buf)
    }
}
