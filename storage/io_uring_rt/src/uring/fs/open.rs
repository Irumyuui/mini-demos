use std::{
    ffi::CString,
    os::{
        fd::{AsRawFd, RawFd},
        unix::ffi::OsStrExt,
    },
    path::Path,
};

use rustix::fs::CWD;
use rustix_uring::{opcode, types};

use crate::uring::op::{CompleteAble, Completion, Op};

use super::{File, OpenOptions, shared_fd::SharedFd};

pub struct Open {
    path: CString,
}

impl Op<Open> {
    pub fn open<P: AsRef<Path>>(path: P, opts: &OpenOptions) -> std::io::Result<Self> {
        let flag = opts.gen_flags()?;
        let path = CString::new(path.as_ref().as_os_str().as_bytes())?;

        Op::submit_with(Open { path }, |open| {
            let ptr = open.path.as_c_str().as_ptr();
            opcode::OpenAt::new(types::Fd(CWD.as_raw_fd()), ptr)
                .flags(flag)
                .mode(opts.mode)
                .build()
        })
    }
}

impl CompleteAble for Open {
    type Output = std::io::Result<File>;

    fn handle_completion(comp: Completion<Self>) -> Self::Output {
        let raw_fd = comp.result?;
        let fd = SharedFd::new(raw_fd as RawFd);
        let file = File::from(fd);
        Ok(file)
    }
}
