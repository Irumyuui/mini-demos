#![allow(unused)]

use std::{
    fs::{create_dir_all, File},
    io::{BufReader, Read, Seek, Write},
};

use fs2::FileExt;

use crate::{FileObject, FileSystem, OpenOptions};

pub trait CrossFileExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;

    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> std::io::Result<()> {
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
}

impl CrossFileExt for File {
    #[cfg(unix)]
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        use std::os::unix::fs::FileExt;
        FileExt::read_at(self, buf, offset)
    }

    #[cfg(windows)]
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        use std::os::windows::fs::FileExt;
        FileExt::seek_read(self, buf, offset)
    }

    #[cfg(not(any(unix, windows)))]
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        unimplemented!()
    }
}

impl FileObject for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Read::read(self, buf)
    }

    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        CrossFileExt::read_at(self, buf, offset)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        BufReader::new(self).read_to_end(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Write::write(self, buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Write::flush(self)
    }

    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Seek::seek(self, pos)
    }

    fn lock(&self) -> std::io::Result<()> {
        FileExt::try_lock_exclusive(self)
    }

    fn unlock(&self) -> std::io::Result<()> {
        FileExt::unlock(self)
    }

    fn sync(&self) -> std::io::Result<()> {
        self.sync_all()
    }

    fn len(&self) -> std::io::Result<u64> {
        Ok(self.metadata()?.len())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NativeFileSystem;

impl FileSystem for NativeFileSystem {
    type File = File;

    fn open<P: AsRef<std::path::Path>>(
        &self,
        options: OpenOptions,
        path: P,
    ) -> std::io::Result<Self::File> {
        let file = std::fs::OpenOptions::new()
            .read(options.read)
            .write(options.write)
            .create(options.create)
            .truncate(options.truncate)
            .append(options.append)
            .open(path)?;
        Ok(file)
    }

    fn remove<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        std::fs::remove_file(path)
    }

    fn remove_dir<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        std::fs::remove_dir(path)
    }

    fn exists<P: AsRef<std::path::Path>>(&self, path: P) -> bool {
        path.as_ref().exists()
    }

    fn create_dir_all<P: AsRef<std::path::Path>>(&self, dir: P) -> std::io::Result<()> {
        create_dir_all(dir)
    }
}
