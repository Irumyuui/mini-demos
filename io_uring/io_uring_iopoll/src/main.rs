use std::{
    ops::{Deref, DerefMut},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
};

use anyhow::Context;
use io_uring::IoUring;

const BUFFER_SIZE: usize = 4096;

#[repr(C, align(4096))]
struct AlignedBuffer(Box<[u8; BUFFER_SIZE]>);

impl Default for AlignedBuffer {
    fn default() -> Self {
        Self(Box::new([0; BUFFER_SIZE]))
    }
}

impl AlignedBuffer {
    fn new() -> Self {
        Self::default()
    }
}

impl Deref for AlignedBuffer {
    type Target = [u8; BUFFER_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AlignedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn main() -> anyhow::Result<()> {
    let mut ring: IoUring = IoUring::builder().setup_iopoll().build(128)?;

    let read_file = std::fs::OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .custom_flags(nix::libc::O_DIRECT)
        .open("./Cargo.toml")?;

    let mut read_buf = AlignedBuffer::new();
    let read_entry = io_uring::opcode::Read::new(
        io_uring::types::Fd(read_file.as_raw_fd()),
        read_buf.as_mut_ptr(),
        read_buf.len() as _,
    )
    .offset(0)
    .build();
    unsafe {
        ring.submission().push(&read_entry)?;
    }

    let read_data = loop {
        ring.submit()?;
        let Some(cqe) = ring.completion().next() else {
            continue;
        };

        if cqe.result() < 0 {
            return Err(std::io::Error::from_raw_os_error(-cqe.result()).into());
        }

        let bytes_read = cqe.result() as usize;
        let read_data = &read_buf[..bytes_read];

        println!("{}", String::from_utf8(read_data.to_vec())?);
        break read_data;
    };

    let write_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .custom_flags(nix::libc::O_DIRECT)
        .open("./io_uring_test_io_poll.txt")
        .with_context(|| "open write")?;

    let write_entry = io_uring::opcode::Write::new(
        io_uring::types::Fd(write_file.as_raw_fd()),
        read_buf.as_ptr(),
        read_buf.len() as _,
    )
    .offset(0)
    .build();
    unsafe {
        ring.submission().push(&write_entry)?;
    }

    loop {
        ring.submit()?;
        let Some(cqe) = ring.completion().next() else {
            continue;
        };

        if cqe.result() < 0 {
            return Err(std::io::Error::from_raw_os_error(-cqe.result()).into());
        }
        break;
    }

    write_file.set_len(read_data.len() as u64)?;
    write_file.sync_all()?;

    Ok(())
}
