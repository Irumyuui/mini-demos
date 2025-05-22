use std::{
    os::fd::{AsRawFd, RawFd},
    path::Path,
};

use crate::uring::{fs::OpenOptions, op::Op, prelude::BufResult};

use super::{AsIoVec, AsIoVecMut, metadata::Metadata, shared_fd::SharedFd};

pub struct File {
    fd: SharedFd,
}

impl File {
    pub async fn read_at<T>(&self, buf: T, offset: u64) -> BufResult<T>
    where
        T: AsIoVecMut,
    {
        let op = Op::read_at(&self.fd, buf, offset).unwrap();
        op.complete().await
    }

    pub async fn write_at<T>(&self, buf: T, offset: u64) -> BufResult<T>
    where
        T: AsIoVec,
    {
        let op = Op::write_at(&self.fd, buf, offset).unwrap();
        op.complete().await
    }

    pub async fn open<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        OpenOptions::new().read(true).open(path).await
    }

    pub async fn create<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await
    }

    pub async fn metadata(&self) -> std::io::Result<Metadata> {
        Op::statx_using_fd(&self.fd)?.complete().await
    }

    pub async fn sync_all(&self) -> std::io::Result<()> {
        Op::sync_all(&self.fd)?.complete().await
    }

    pub async fn sync_data(&self) -> std::io::Result<()> {
        Op::sync_data(&self.fd)?.complete().await
    }

    /// # Safety
    ///
    /// This function is unsafe because it takes a raw file descriptor and creates a `File` instance from it.
    ///
    /// Requies the caller to ensure that the file descriptor is valid and not already closed.
    pub unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let fd = SharedFd::new(fd);
        Self { fd }
    }

    pub fn from_std_fd(fd: std::fs::File) -> Self {
        let raw_fd = fd.as_raw_fd();
        std::mem::forget(fd);
        unsafe { Self::from_raw_fd(raw_fd) }
    }

    pub async fn close(mut self) -> std::io::Result<()> {
        self.fd.close().await;
        Ok(())
    }
}

impl From<SharedFd> for File {
    fn from(fd: SharedFd) -> Self {
        Self { fd }
    }
}

#[cfg(test)]
mod tests {
    use core::slice;
    use std::{
        alloc::Layout,
        io::{Read, Write},
        marker::PhantomData,
        ops::{Deref, DerefMut},
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::MetadataExt,
        },
        ptr::NonNull,
        vec,
    };

    use rustix::fs::OFlags;
    use static_assertions::assert_impl_all;
    use tempfile::tempfile;

    use crate::uring::{
        fs::{OpenOptions, shared_fd::SharedFd},
        rt::{Runtime, default_rt},
    };

    use super::File;

    const ALIGNED: usize = 4096; // default aligned

    /// For direce io.
    struct AlignedBuffer {
        ptr: NonNull<u8>,
        /// io uring buf 最大大小只有 `u32::MAX`，不要超过这个界限
        len: usize,

        /// Aligned buffer 就像一个 `Box<[u8]>`，但是它的内存是对齐的，并且保有自己的所有权
        _marker: PhantomData<Box<[u8]>>,
    }

    impl Deref for AlignedBuffer {
        type Target = [u8];

        fn deref(&self) -> &Self::Target {
            unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
        }
    }

    impl AsRef<[u8]> for AlignedBuffer {
        fn as_ref(&self) -> &[u8] {
            self
        }
    }

    impl DerefMut for AlignedBuffer {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
        }
    }

    impl AsMut<[u8]> for AlignedBuffer {
        fn as_mut(&mut self) -> &mut [u8] {
            self
        }
    }

    // 即使 io 上下文不支持 Send/Sync ，但 buffer 依然满足该条件
    unsafe impl Send for AlignedBuffer {}
    unsafe impl Sync for AlignedBuffer {}

    impl AlignedBuffer {
        fn new(size: usize) -> Self {
            let layout = Layout::from_size_align(size, ALIGNED).unwrap();

            let ptr = unsafe { std::alloc::alloc(layout) };
            assert!(!ptr.is_null(), "Failed to allocate memory");
            assert!(ptr.is_aligned(), "Memory is not aligned");

            Self {
                ptr: unsafe { NonNull::new_unchecked(ptr) },
                len: size,
                _marker: PhantomData,
            }
        }
    }

    impl Drop for AlignedBuffer {
        fn drop(&mut self) {
            let layout = Layout::from_size_align(self.len, ALIGNED).unwrap();
            unsafe { std::alloc::dealloc(self.ptr.as_ptr(), layout) };
        }
    }

    #[test]
    fn test_file_close() {
        let path = tempfile::tempdir().unwrap();
        let file_path = path.path().join("1.txt");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let rt = default_rt().unwrap();
        let p = file_path.clone();
        rt.block_on(async move {
            let file = File::open(p).await.unwrap();
            file.sync_data().await.unwrap();
            drop(file);
        });

        let mut file = std::fs::File::open(file_path).unwrap();
        let mut buf = vec![0_u8; 11];
        file.read_exact(&mut buf).unwrap();
        assert_eq!(&buf[..], b"hello world");
    }

    #[test]
    fn test_read_at() {
        let path = tempfile::tempdir().unwrap();

        let file_path = path.path().join("1.txt");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let rt = default_rt().unwrap();
        let file = std::fs::File::open(file_path).unwrap();

        rt.block_on(async move {
            let fd = file.as_raw_fd();
            std::mem::forget(file);
            let shared_fd = SharedFd::new(fd);

            let file = File { fd: shared_fd };

            let buf = vec![0_u8; 11];
            let (res, buf) = file.read_at(buf, 0).await;

            let res = res.unwrap();
            assert_eq!(res, 11);
            assert_eq!(&buf[..], b"hello world");
        });
    }

    #[test]
    fn test_write_at() {
        let path = tempfile::tempdir().unwrap();

        let file_path = path.path().join("2.txt");
        let rt = default_rt().unwrap();
        let file = std::fs::File::create(&file_path).unwrap();

        rt.block_on(async move {
            let fd = file.as_raw_fd();
            std::mem::forget(file);
            let shared_fd = SharedFd::new(fd);

            let file = File { fd: shared_fd };

            let buf = b"hello world".to_vec();
            let (res, buf) = file.write_at(buf, 0).await;
            let res = res.unwrap();
            assert_eq!(res, 11);
            assert_eq!(&buf[..], b"hello world");
        });

        let mut file = std::fs::File::open(file_path).unwrap();
        let mut buf = vec![0_u8; 11];
        file.read_exact(&mut buf).unwrap();
        assert_eq!(&buf[..], b"hello world");
    }

    #[test]
    fn test_open() {
        let path = tempfile::tempdir().unwrap();

        let file_path = path.path().join("3.txt");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let rt = default_rt().unwrap();
        rt.block_on(async move {
            let file = File::open(file_path).await.unwrap();
            let buf = vec![0_u8; 11];
            let (res, buf) = file.read_at(buf, 0).await;
            let res = res.unwrap();
            assert_eq!(res, 11);
            assert_eq!(&buf[..], b"hello world");
        });
    }

    #[test]
    fn test_create_and_write() {
        let path = tempfile::tempdir().unwrap();

        let file_path = path.path().join("4.txt");
        let p = file_path.clone();
        let rt = default_rt().unwrap();
        rt.block_on(async move {
            let file = File::create(p).await.unwrap();
            let buf = b"hello world".to_vec();
            let (res, buf) = file.write_at(buf, 0).await;
            let res = res.unwrap();
            assert_eq!(res, 11);
            assert_eq!(&buf[..], b"hello world");
        });

        let mut file = std::fs::File::open(file_path).unwrap();
        let mut buf = vec![0_u8; 11];
        file.read_exact(&mut buf).unwrap();
        assert_eq!(&buf[..], b"hello world");
    }

    #[test]
    fn test_metadata() {
        let path = tempfile::tempdir().unwrap();

        let file_path = path.path().join("5.txt");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let rt = default_rt().unwrap();
        rt.block_on(async move {
            let file = File::open(file_path).await.unwrap();
            let metadata = file.metadata().await.unwrap();
            assert_eq!(metadata.size(), 11);
        });
    }

    #[test]
    fn test_read_cargo_lock() {
        default_rt().unwrap().block_on(async {
            let path = "Cargo.lock";
            let file = File::open(path).await.unwrap();

            let len = file.metadata().await.unwrap().size();
            let buf = vec![0_u8; len as usize];
            let (res, buf) = file.read_at(buf, 0).await;
            let res = res.unwrap();
            assert_eq!(res, len as usize);

            eprintln!("buf: {:?}", String::from_utf8(buf).unwrap());
        });
    }

    #[test]
    fn test_file_len() {
        default_rt().unwrap().block_on(async move {
            let path = tempfile::tempdir().unwrap();
            let file_path = path.path().join("6.txt");
            std::fs::File::create(&file_path)
                .unwrap()
                .write_all(b"hello world")
                .unwrap();

            let file = File::open(file_path).await.unwrap();
            assert_eq!(file.metadata().await.unwrap().size(), 11);
        });
    }

    #[test]
    fn test_sync_data() {
        let path = tempfile::tempdir().unwrap();
        let file_path = path.path().join("7.txt");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let rt = default_rt().unwrap();
        rt.block_on(async move {
            let file = File::open(file_path).await.unwrap();
            file.sync_data().await.unwrap();
            drop(file);
        });
    }

    #[test]
    fn test_sync_all() {
        let path = tempfile::tempdir().unwrap();
        let file_path = path.path().join("7.txt");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let rt = default_rt().unwrap();
        rt.block_on(async move {
            let file = File::open(file_path).await.unwrap();
            file.sync_all().await.unwrap();
            drop(file);
        });
    }

    #[test]
    fn test_direct_io() {
        let path = tempfile::tempdir().unwrap();
        let file_path = path.path().join("8.txt");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let rt = default_rt().unwrap();
        rt.block_on(async move {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .custom_flags(OFlags::DIRECT)
                .open(file_path)
                .await
                .unwrap();

            let buf = AlignedBuffer::new(ALIGNED);
            let (res, buf) = file.read_at(buf, 0).await;
            let res = res.unwrap();

            assert_eq!(res, 11);
            assert_eq!(&buf[..res], b"hello world");
        });
    }

    #[test]
    fn test_fd_closed() {
        let mut tempfile = tempfile::NamedTempFile::new().unwrap();
        tempfile.write_all(b"hello world").unwrap();
        tempfile.as_file_mut().sync_data().unwrap();

        let rt = default_rt().unwrap();
        rt.block_on(async move {
            let file = File::open(tempfile.path()).await.unwrap();
            let fd = file.fd.raw_fd();
            file.close().await.unwrap();

            let raw_meta = tempfile.as_file().metadata().unwrap();

            let file = unsafe { std::fs::File::from_raw_fd(fd) };
            let meta = file.metadata();
            std::mem::forget(file);

            if let Ok(meta) = meta {
                if meta.is_file() {
                    let inode = meta.ino();
                    let actual = raw_meta.ino();
                    assert_ne!(inode, actual);
                }
            }
        });
    }
}
