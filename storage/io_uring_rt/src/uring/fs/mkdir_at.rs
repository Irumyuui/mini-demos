use std::{
    ffi::CString,
    future::poll_fn,
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::Path,
    pin::Pin,
    task::{Poll, ready},
};

use rustix::fs::{CWD, Mode};
use rustix_uring::{opcode, types};

use crate::uring::op::{CompleteAble, Completion, Op};

pub struct Mkdir {
    pub(crate) path: CString,
}

impl Op<Mkdir> {
    pub fn mkdir<P: AsRef<Path>>(path: P, mode: Mode) -> std::io::Result<Self> {
        let path = CString::new(path.as_ref().as_os_str().as_bytes())?;

        Op::submit_with(Mkdir { path }, |mkdir| {
            let ptr = mkdir.path.as_c_str().as_ptr();
            opcode::MkDirAt::new(types::Fd(CWD.as_raw_fd()), ptr)
                .mode(mode)
                .build()
        })
    }
}

impl CompleteAble for Mkdir {
    type Output = std::io::Result<()>;

    fn handle_completion(comp: Completion<Self>) -> Self::Output {
        comp.result.map(|_| ())
    }
}
