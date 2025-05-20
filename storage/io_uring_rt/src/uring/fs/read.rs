use std::{future::poll_fn, pin::Pin, task::ready};

use rustix_uring::{opcode, types};

use super::{AsIoVecMut, shared_fd::SharedFd};
use crate::uring::{
    op::{CompleteAble, Completion, Op},
    prelude::BufResult,
};

pub struct Read<T> {
    fd: SharedFd,
    buf: Option<T>,
}

impl<T> Read<T> {
    fn new(fd: SharedFd, buf: T) -> Self {
        Self { fd, buf: Some(buf) }
    }
}

impl<T> Op<Read<T>>
where
    T: AsIoVecMut,
{
    pub fn read_at(fd: &SharedFd, buf: T, offset: u64) -> std::io::Result<Self> {
        Op::submit_with(Read::new(fd.clone(), buf), |read| {
            let fd = read.fd.raw_fd();
            let (ptr, len) = read.buf.as_mut().unwrap().as_io_vec_mut();

            opcode::Read::new(types::Fd(fd), ptr, len as _)
                .offset(offset)
                .build()
        })
    }
}

impl<T> CompleteAble for Read<T>
where
    T: AsIoVecMut,
{
    type Output = BufResult<T>;

    fn handle_completion(mut comp: Completion<Self>) -> Self::Output {
        let res = comp.result.map(|res| res as usize);
        let buf = comp.data.buf.take().unwrap();
        (res, buf)
    }
}
