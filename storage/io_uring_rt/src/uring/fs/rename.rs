use std::{
    ffi::CString,
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::Path,
};

use rustix::fs::CWD;
use rustix_uring::{opcode, types};

use crate::uring::op::{CompleteAble, Op};

#[derive(Debug)]
pub struct Rename {
    from: CString,
    to: CString,
}

impl Op<Rename> {
    pub fn rename<P, Q>(from: P, to: Q) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let from = CString::new(from.as_ref().as_os_str().as_bytes())?;
        let to = CString::new(to.as_ref().as_os_str().as_bytes())?;

        let rename = Rename { from, to };
        Op::submit_with(rename, |rename| {
            let from_ptr = rename.from.as_ptr();
            let to_ptr = rename.to.as_ptr();

            opcode::RenameAt::new(
                types::Fd(CWD.as_raw_fd()),
                from_ptr,
                types::Fd(CWD.as_raw_fd()),
                to_ptr,
            )
            .build()
        })
    }
}

impl CompleteAble for Rename {
    type Output = std::io::Result<()>;

    fn handle_completion(comp: crate::uring::op::Completion<Self>) -> Self::Output {
        comp.result.map(|_| ())
    }
}
