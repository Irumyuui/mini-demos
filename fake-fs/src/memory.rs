#![allow(unused)]

use std::{
    collections::BTreeMap,
    io::{Cursor, Read, Seek, Write},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc, RwLock},
};

use crate::{FileObject, FileSystem};

pub struct MemoryFileSystem {
    fs: Arc<RwLock<FileNode>>,
}

impl FileSystem for MemoryFileSystem {
    type File = MemoryFileInner;

    fn open<P: AsRef<std::path::Path>>(
        &self,
        options: crate::prelude::OpenOptions,
        path: P,
    ) -> std::io::Result<Self::File> {
        let mut root_dir = BTreeMap::new();
        let mut cur = Some(root_dir);
        for path in std::env::split_paths(path.as_ref()) {
            cur.as_mut().unwrap().entry(path).or_insert(None);
        }

        todo!()
    }

    fn remove<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        todo!()
    }

    fn remove_dir<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        todo!()
    }

    fn create_dir_all<P: AsRef<std::path::Path>>(&self, dir: P) -> std::io::Result<()> {
        todo!()
    }

    fn exists<P: AsRef<std::path::Path>>(&self, path: P) -> bool {
        todo!()
    }
}

pub enum FileNode {
    Dir(PathBuf, Option<BTreeMap<PathBuf, FileNode>>),
    File(PathBuf, MemoryFile),
}

pub struct MemoryFile {
    inner: Arc<RwLock<MemoryFileInner>>,
}

impl Default for MemoryFile {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MemoryFileInner::default())),
        }
    }
}

impl FileObject for MemoryFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.write().unwrap().read(buf)
    }

    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        self.inner.write().unwrap().read_at(buf, offset)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.inner.write().unwrap().read_to_end(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.write().unwrap().flush()
    }

    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.inner.write().unwrap().seek(pos)
    }

    fn lock(&self) -> std::io::Result<()> {
        self.inner.read().unwrap().lock()
    }

    fn unlock(&self) -> std::io::Result<()> {
        self.inner.read().unwrap().unlock()
    }

    fn sync(&self) -> std::io::Result<()> {
        self.inner.read().unwrap().sync()
    }

    fn len(&self) -> std::io::Result<u64> {
        self.inner.read().unwrap().len()
    }
}

pub struct MemoryFileInner {
    data: Cursor<Vec<u8>>,
    locked: AtomicBool,
}

impl Default for MemoryFileInner {
    fn default() -> Self {
        Self {
            data: Cursor::new(Vec::new()),
            locked: AtomicBool::new(false),
        }
    }
}

impl FileObject for MemoryFileInner {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.data.read(buf)
    }

    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let data = self.data.get_ref();
        let len = data.len() as u64;
        if offset >= len {
            return Ok(0);
        }
        if buf.len() as u64 + offset > len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "End of file",
            ));
        }

        buf.copy_from_slice(data[offset as usize..offset as usize + buf.len()].as_ref());
        Ok(buf.len())
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.data.set_position(0);
        self.data.read_to_end(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let pos = self.data.position();
        self.data.set_position(self.data.get_ref().len() as u64);
        let res = self.data.write(buf);
        self.data.set_position(pos);
        res
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.data.seek(pos)
    }

    fn lock(&self) -> std::io::Result<()> {
        if self
            .locked
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_err()
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "File is already locked",
            ));
        }

        Ok(())
    }

    fn unlock(&self) -> std::io::Result<()> {
        self.locked
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn sync(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn len(&self) -> std::io::Result<u64> {
        Ok(self.data.get_ref().len() as u64)
    }
}
