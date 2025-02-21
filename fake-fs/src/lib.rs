use std::path::Path;

use options::OpenOptions;

mod options;
pub mod memory;
pub mod native;

pub mod prelude {
    pub use crate::native::NativeFileSystem;
    pub use crate::options::OpenOptions;
    pub use crate::FileSystem;
}

pub trait FileObject: Send + Sync {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;

    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;

    fn read_exact_at(&mut self, mut buf: &mut [u8], mut offset: u64) -> std::io::Result<()> {
        while !buf.is_empty() {
            match self.read_at(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &mut buf[n..];
                    offset += n as u64;
                }
                Err(e) => {
                    if matches!(e.kind(), std::io::ErrorKind::Interrupted) {
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        if buf.is_empty() {
            return Ok(());
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "failed to fill whole buffer",
        ))
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize>;

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;

    fn flush(&mut self) -> std::io::Result<()>;

    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64>;

    fn lock(&self) -> std::io::Result<()>;

    fn unlock(&self) -> std::io::Result<()>;

    fn sync(&self) -> std::io::Result<()>;

    fn len(&self) -> std::io::Result<u64>;
}

pub trait FileSystem: Send + Sync {
    type File: FileObject;

    fn open<P: AsRef<Path>>(&self, options: OpenOptions, path: P) -> std::io::Result<Self::File>;

    fn remove<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()>;

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()>;

    fn create_dir_all<P: AsRef<Path>>(&self, dir: P) -> std::io::Result<()>;

    fn exists<P: AsRef<Path>>(&self, path: P) -> bool;
}
