use std::{
    ffi::CString,
    future::poll_fn,
    io,
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::Path,
    pin::Pin,
    task::{Poll, ready},
};

use rustix_uring::{opcode, types};

use crate::uring::op::{CompleteAble, Completion, Op};

pub struct UnlinkAt {
    path: CString,
}

impl Op<UnlinkAt> {
    pub fn unlink_dir<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::unlink_at(path, rustix::fs::AtFlags::REMOVEDIR)
    }

    pub fn unlink_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::unlink_at(path, rustix::fs::AtFlags::empty())
    }

    fn unlink_at<P: AsRef<Path>>(path: P, flags: rustix::fs::AtFlags) -> io::Result<Self> {
        let path = CString::new(path.as_ref().as_os_str().as_bytes())?;
        Op::submit_with(UnlinkAt { path }, |unlink| {
            let ptr = unlink.path.as_c_str().as_ptr();
            opcode::UnlinkAt::new(types::Fd(rustix::fs::CWD.as_raw_fd()), ptr)
                .flags(flags)
                .build()
        })
    }
}

impl CompleteAble for UnlinkAt {
    type Output = std::io::Result<()>;

    fn handle_completion(comp: Completion<Self>) -> Self::Output {
        comp.result.map(|_| ())
    }
}
