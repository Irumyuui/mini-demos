use std::path::Path;

use rustix::fs::{Mode, OFlags};

use crate::uring::op::Op;

use super::file::File;

#[derive(Debug, Clone)]
pub struct OpenOptions {
    // generic
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,

    // system-specific
    custom_flags: OFlags,
    pub(crate) mode: Mode,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenOptions {
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,

            custom_flags: OFlags::empty(),
            mode: Mode::from_bits(0o666).unwrap(), // Mode::RUSR | Mode::WUSR | Mode::RGRP | Mode::WGRP | Mode::ROTH | Mode::WOTH,
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    pub fn custom_flags(&mut self, flags: OFlags) -> &mut Self {
        self.custom_flags = flags;
        self
    }

    pub fn mode(&mut self, mode: Mode) -> &mut Self {
        self.mode = mode;
        self
    }

    pub async fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<File> {
        Op::open(path, self)?.complete().await
    }

    fn get_access_mode(&self) -> std::io::Result<OFlags> {
        match (self.read, self.write, self.append) {
            (true, false, false) => Ok(OFlags::RDONLY),
            (false, true, false) => Ok(OFlags::WRONLY),
            (true, true, false) => Ok(OFlags::RDWR),
            (false, _, true) => Ok(OFlags::WRONLY | OFlags::APPEND),
            (true, _, true) => Ok(OFlags::RDWR | OFlags::APPEND),
            (false, false, false) => Err(std::io::Error::from(rustix::io::Errno::INVAL)),
        }
    }

    fn get_creation_mode(&self) -> std::io::Result<OFlags> {
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return Err(std::io::Error::from(rustix::io::Errno::INVAL));
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return Err(std::io::Error::from(rustix::io::Errno::INVAL));
                }
            }
        }

        Ok(match (self.create, self.truncate, self.create_new) {
            (false, false, false) => OFlags::empty(),
            (true, false, false) => OFlags::CREATE,
            (false, true, false) => OFlags::TRUNC,
            (true, true, false) => OFlags::CREATE | OFlags::TRUNC,
            (_, _, true) => OFlags::CREATE | OFlags::EXCL,
        })
    }

    pub(crate) fn gen_flags(&self) -> std::io::Result<OFlags> {
        // let flags = OFlags::CLOEXEC
        //     | self.get_access_mode()?
        //     | self.get_creation_mode()?
        //     | (self.custom_flags & !OFlags::ACCMODE);
        // Ok(flags)
        let access = self.get_access_mode()?;
        let creation = self.get_creation_mode()?;
        let custom = self.custom_flags & !OFlags::ACCMODE;
        let flags = OFlags::CLOEXEC | access | creation | custom;
        // eprintln!(
        //     "Flags: access={:?}, creation={:?}, custom={:?}, combined={:?}",
        //     access, creation, custom, flags
        // );
        Ok(flags)
    }
}
