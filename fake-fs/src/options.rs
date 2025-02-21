use std::path::Path;

use crate::FileSystem;

#[derive(Debug, Clone)]
pub struct OpenOptions {
    pub(crate) read: bool,
    pub(crate) write: bool,
    pub(crate) create: bool,
    pub(crate) truncate: bool,
    pub(crate) append: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            read: false,
            write: false,
            create: false,
            truncate: false,
            append: false,
        }
    }
}

impl OpenOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self) -> &mut Self {
        self.read = true;
        self
    }

    pub fn write(&mut self) -> &mut Self {
        self.write = true;
        self
    }

    pub fn create(&mut self) -> &mut Self {
        self.create = true;
        self
    }

    pub fn truncate(&mut self) -> &mut Self {
        self.truncate = true;
        self
    }

    pub fn append(&mut self) -> &mut Self {
        self.append = true;
        self
    }

    pub fn open<FS: FileSystem, P: AsRef<Path>>(
        self,
        fs: &FS,
        path: P,
    ) -> std::io::Result<FS::File> {
        fs.open(self, path)
    }
}
