mod close;
mod cread_dir_all;
mod file;
mod fsync;
mod metadata;
mod mkdir_at;
mod open;
mod open_options;
mod read;
mod removed;
mod rename;
mod write;

pub(crate) mod shared_fd;

use std::{future::poll_fn, path::Path, pin::Pin};

pub use file::File;
pub use open_options::OpenOptions;
use rustix::fs::Mode;

use super::op::{Completion, Op};

pub trait AsIoVec: Unpin + 'static {
    fn as_io_vec(&self) -> (*mut u8, usize);

    fn as_slice(&self) -> &[u8] {
        let (ptr, len) = self.as_io_vec();
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

impl<T> AsIoVec for T
where
    T: AsRef<[u8]> + Unpin + 'static,
{
    fn as_io_vec(&self) -> (*mut u8, usize) {
        let slice = self.as_ref();
        let ptr = slice.as_ptr() as *mut u8;
        let len = slice.len();
        (ptr, len)
    }
}

pub trait AsIoVecMut: AsIoVec {
    fn as_io_vec_mut(&mut self) -> (*mut u8, usize) {
        let (ptr, len) = self.as_io_vec();
        (ptr, len)
    }

    fn as_slice_mut(&mut self) -> &mut [u8] {
        let (ptr, len) = self.as_io_vec_mut();
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }
}

impl<T> AsIoVecMut for T where T: AsMut<[u8]> + AsIoVec {}

pub async fn mkdir<P>(path: P) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    Op::mkdir(&path, rustix::fs::Mode::from(0o777))?
        .complete()
        .await
}

pub async fn create_dir_all<P>(path: P) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    cread_dir_all::DirBuilder::new()
        .recursive(true)
        .create(path)
        .await
}

pub async fn remove_dir<P>(path: P) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    Op::unlink_dir(&path)?.complete().await
}

pub async fn remove_file<P>(path: P) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    Op::unlink_file(&path)?.complete().await
}

pub async fn rename<P, Q>(from: P, to: Q) -> std::io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    Op::rename(from, to)?.complete().await
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;
    use tokio::fs::{remove_dir, remove_file};

    use crate::uring::{fs::rename, rt::default_rt};

    use super::{create_dir_all, mkdir};

    #[test]
    fn test_mkdir() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("path");

        default_rt().unwrap().block_on(async move {
            mkdir(&path).await.unwrap();
            assert!(path.is_dir());
        });
    }

    #[test]
    fn test_create_dir_all() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("path").join("to").join("dir");

        default_rt().unwrap().block_on(async move {
            create_dir_all(&path).await.unwrap();
            assert!(dir.path().join("path").is_dir());
            assert!(dir.path().join("path").join("to").is_dir());
            assert!(dir.path().join("path").join("to").join("dir").is_dir());
        });
    }

    #[test]
    fn test_remove_file() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("test.txt");

        default_rt().unwrap().block_on(async move {
            assert!(!path.exists());
            std::fs::File::create(&path).unwrap();
            assert!(path.exists());

            remove_file(&path).await.unwrap();
            assert!(!path.exists());
        });
    }

    #[test]
    fn test_remove_dir() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("test.txt");

        default_rt().unwrap().block_on(async move {
            assert!(!path.exists());
            std::fs::create_dir(&path).unwrap();
            assert!(path.exists());

            remove_dir(&path).await.unwrap();
            assert!(!path.exists());
        });
    }

    #[test]
    fn test_create_existing_dir() {
        default_rt().unwrap().block_on(async move {
            let tempdir = tempdir().unwrap();
            let res = create_dir_all(tempdir.path()).await;
            assert!(res.is_ok());
        });
    }

    #[test]
    fn test_create_dir_with_emoji() {
        // emoji..
        // but ojbk
        default_rt().unwrap().block_on(async move {
            let tempdir = tempdir().unwrap();

            let path = tempdir.path().join("test.txt");
            create_dir_all(&path).await.unwrap();
            assert!(path.is_dir());

            let path = tempdir.path().join("😊");
            create_dir_all(&path).await.unwrap();
            assert!(path.is_dir());
        });
    }

    #[test]
    fn test_create_invalid_path_dir() {
        default_rt().unwrap().block_on(async move {
            let tempdir = tempdir().unwrap();

            // just zero name, must be invalid
            let path = tempdir.path().join("\0\0\0");
            let res = create_dir_all(&path).await;
            assert!(res.is_err());
        });
    }

    #[test]
    fn test_create_dir_on_existing_file() {
        default_rt().unwrap().block_on(async move {
            let tempdir = tempdir().unwrap();
            let filepath = tempdir.path().join("test.txt");

            std::fs::File::create(&filepath).unwrap();

            let res = create_dir_all(&filepath).await;
            assert!(res.is_err());
        });
    }

    #[test]
    fn test_create_long_path_dir() {
        default_rt().unwrap().block_on(async move {
            let tempdir = tempdir().unwrap();

            let mut path = tempdir.path().join("test");
            for _ in 0..114 {
                path.push("path/");
            }

            let res = create_dir_all(&path).await;
            assert!(res.is_ok());
        });
    }

    #[test]
    fn test_rename() {
        default_rt().unwrap().block_on(async {
            let tempdir = tempdir().unwrap();

            let path = tempdir.path().join("test.txt");
            assert!(!path.exists());
            std::fs::File::create(&path).unwrap();
            assert!(path.exists());
            assert!(path.is_file());

            let new_path = tempdir.path().join("test2.txt");
            assert!(!new_path.exists());
            rename(&path, &new_path).await.unwrap();
            assert!(!path.exists());
            assert!(new_path.exists());
            assert!(new_path.is_file());
        });
    }
}
