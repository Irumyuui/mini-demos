use std::{future::poll_fn, pin::Pin, task::Poll};

use rustix_uring::{opcode, types};

use crate::uring::op::{CompleteAble, Op};

use super::shared_fd::SharedFd;

pub struct Fsync {
    fd: SharedFd,
}

impl Op<Fsync> {
    pub fn sync_all(fd: &SharedFd) -> std::io::Result<Self> {
        Self::fsync(fd)
    }

    fn fsync(fd: &SharedFd) -> std::io::Result<Self> {
        let data = Fsync { fd: fd.clone() };

        Op::submit_with(data, |fsync| {
            opcode::Fsync::new(rustix_uring::types::Fd(fsync.fd.raw_fd())).build()
        })
    }

    pub fn sync_data(fd: &SharedFd) -> std::io::Result<Self> {
        Self::fdatasync(fd)
    }

    fn fdatasync(fd: &SharedFd) -> std::io::Result<Self> {
        let data = Fsync { fd: fd.clone() };

        Op::submit_with(data, |fsync| {
            opcode::Fsync::new(rustix_uring::types::Fd(fsync.fd.raw_fd()))
                .flags(types::FsyncFlags::DATASYNC)
                .build()
        })
    }
}

impl CompleteAble for Fsync {
    type Output = std::io::Result<()>;

    fn handle_completion(comp: crate::uring::op::Completion<Self>) -> Self::Output {
        comp.result.map(|_| ())
    }
}
