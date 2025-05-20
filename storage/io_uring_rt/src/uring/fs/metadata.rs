use std::mem::MaybeUninit;

use rustix::fs::AtFlags;
use rustix_uring::{opcode, types};

use crate::uring::op::{CompleteAble, Op};

use super::shared_fd::SharedFd;

pub struct Metadata {
    attr: rustix::fs::Statx,
}

impl Metadata {
    pub fn size(&self) -> u64 {
        self.attr.stx_size
    }

    pub fn is_dir(&self) -> bool {
        rustix::fs::FileType::Directory
            == rustix::fs::FileType::from_raw_mode(self.attr.stx_mode as u32)
    }

    pub fn is_file(&self) -> bool {
        rustix::fs::FileType::RegularFile
            == rustix::fs::FileType::from_raw_mode(self.attr.stx_mode as u32)
    }

    pub fn is_syslink(&self) -> bool {
        rustix::fs::FileType::Symlink
            == rustix::fs::FileType::from_raw_mode(self.attr.stx_mode as u32)
    }
}

pub(crate) struct Statx {
    fd: SharedFd,
    buf: Box<MaybeUninit<rustix::fs::Statx>>,
}

impl Op<Statx> {
    pub(crate) fn statx_using_fd(fd: &SharedFd) -> std::io::Result<Self> {
        let flags = AtFlags::STATX_SYNC_AS_STAT | AtFlags::EMPTY_PATH;
        let statx = Statx {
            fd: fd.clone(),
            buf: Box::new(MaybeUninit::uninit()),
        };

        Op::submit_with(statx, |statx| {
            let fd = statx.fd.raw_fd();
            let statx_buf = statx.buf.as_mut_ptr();

            opcode::Statx::new(types::Fd(fd), c"".as_ptr(), statx_buf)
                .flags(flags)
                .build()
        })
    }
}

impl CompleteAble for Statx {
    type Output = std::io::Result<Metadata>;

    fn handle_completion(comp: crate::uring::op::Completion<Self>) -> Self::Output {
        let res = comp.result?;
        let data = comp.data;
        let attr = unsafe { *data.buf.assume_init() };
        Ok(Metadata { attr })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::uring::{fs::File, rt::default_rt};

    #[test]
    fn test_meta_is_dir() {
        let tempdir = tempdir().unwrap();

        default_rt().unwrap().block_on(async move {
            let path = tempdir.path().join("test_dir");
            std::fs::create_dir(&path).unwrap();

            let std_file = std::fs::File::open(&path).unwrap();
            let meta = std_file.metadata().unwrap();
            assert!(meta.is_dir());

            let fd = File::from_std_fd(std_file);
            let meta = fd.metadata().await.unwrap();

            assert!(meta.is_dir());
        });
    }

    #[test]
    fn test_meta_is_file() {
        let tempdir = tempdir().unwrap();

        default_rt().unwrap().block_on(async move {
            let path = tempdir.path().join("test_file.txt");
            std::fs::File::create(&path).unwrap();

            let std_file = std::fs::File::open(&path).unwrap();
            let meta = std_file.metadata().unwrap();
            assert!(meta.is_file());

            let fd = File::from_std_fd(std_file);
            let meta = fd.metadata().await.unwrap();

            assert!(meta.is_file());
        });
    }
}
