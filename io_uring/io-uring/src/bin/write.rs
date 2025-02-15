use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    os::fd::AsRawFd,
};

use anyhow::ensure;
use io_uring::{opcode, types, IoUring};
use io_uring_example::calc_time;

const CHUNK_SIZE: usize = 1024 * 1024 * 128;
const TOTAL_SIZE: usize = 1024 * 1024 * 1024;
const QUEUE_DEPTH: u32 = 1024;

fn main() -> anyhow::Result<()> {
    calc_time!("io_uring_exp", { io_uring_exp() })?;
    calc_time!("block_fs", { block_fs() })?;

    Ok(())
}

fn io_uring_exp() -> anyhow::Result<()> {
    let mut ring = IoUring::new(QUEUE_DEPTH)?;

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(true)
        .open("read-write.txt")?;

    let raw_fd = file.as_raw_fd();

    let total_chunks = (TOTAL_SIZE + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let data = vec![b'a'; CHUNK_SIZE];
    for i in 0..total_chunks {
        let offset = i * CHUNK_SIZE;
        let write_entry = opcode::Write::new(types::Fd(raw_fd), data.as_ptr(), CHUNK_SIZE as _)
            .offset(offset as _)
            .build()
            .user_data(i as _);

        unsafe {
            ring.submission().push(&write_entry)?;
        }
    }

    ring.submit_and_wait(total_chunks as _)?;

    // 读取部分开始
    let read_file = OpenOptions::new().read(true).open("read-write.txt")?;
    let read_fd = read_file.as_raw_fd();

    let mut read_bufs: Vec<_> = Vec::with_capacity(total_chunks);
    for _ in 0..total_chunks {
        read_bufs.push(vec![0u8; CHUNK_SIZE]);
    }

    for i in 0..total_chunks {
        let offset = i * CHUNK_SIZE;
        let read_entry = opcode::Read::new(
            types::Fd(read_fd),
            read_bufs[i].as_mut_ptr(),
            CHUNK_SIZE as _,
        )
        .offset(offset as _)
        .build()
        .user_data(i as _);

        unsafe {
            ring.submission().push(&read_entry)?;
        }
    }

    ring.submit_and_wait(total_chunks as _)?;

    while let Some(cqe) = ring.completion().next() {
        if cqe.result() >= 0 {
            // let chunk_index = cqe.user_data() as usize;
            let bytes_read = cqe.result();
            ensure!(bytes_read >= 0, "read error");
        }
    }

    for buf in read_bufs.into_iter() {
        ensure!(buf.iter().all(|&b| b == b'a'), "read error");
    }

    println!("Ok");

    drop(file);
    std::fs::remove_file("read-write.txt")?;

    Ok(())
}

fn block_fs() -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(true)
        .open("read-write.txt")?;

    let total_data = vec![b'a'; TOTAL_SIZE];
    file.write_all(&total_data)?;

    file.seek(SeekFrom::Start(0))?;

    let mut read_buf = vec![0u8; TOTAL_SIZE];
    let res = file.read(&mut read_buf)?;
    ensure!(res == TOTAL_SIZE, "read error");
    ensure!(read_buf.iter().all(|&b| b == b'a'), "read error");

    std::fs::remove_file("read-write.txt")?;

    println!("Ok");

    Ok(())
}
