use std::{
    io::{Read, Seek, SeekFrom, Write},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
};

use criterion::{Criterion, criterion_group, criterion_main};
use io_uring::types;
use scopeguard::defer;

fn gen_file<P: AsRef<Path>>(path: P, size: usize) -> std::io::Result<std::fs::File> {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let data = (0..size)
        .map(|i| u8::try_from(i % 256).unwrap())
        .collect::<Vec<_>>();
    file.write_all(&data)?;

    Ok(file)
}

const BLOCK_SIZE: usize = 4096;
const BLOCK_COUNT: usize = 1024;

fn std_read_big_file() -> std::io::Result<()> {
    let dir_path = PathBuf::from("data");
    defer! {
        let _ = std::fs::remove_dir_all(&dir_path);
    }

    std::fs::create_dir(&dir_path)?;
    let file_path = dir_path.join("std_file");

    let mut file = gen_file(&file_path, BLOCK_SIZE * BLOCK_COUNT)?;
    let mut buf = vec![0; BLOCK_SIZE];

    for i in 0..BLOCK_COUNT {
        let offset = i * BLOCK_SIZE;
        file.seek(SeekFrom::Start(offset as _))?;
        file.read_exact(&mut buf)?;
    }

    Ok(())
}

fn uring_read_big_file() -> std::io::Result<()> {
    let dir_path = PathBuf::from("data");
    defer! {
        let _ = std::fs::remove_dir_all(&dir_path);
    }

    std::fs::create_dir(&dir_path)?;
    let file_path = dir_path.join("uring_file");

    let file = gen_file(&file_path, BLOCK_SIZE * BLOCK_COUNT)?;
    let mut bufs = vec![];

    let mut uring = io_uring::IoUring::new(4096)?;

    for i in 0..BLOCK_COUNT {
        let offset = i * BLOCK_SIZE;
        let mut buf = vec![0; BLOCK_SIZE];

        let ptr = buf.as_mut_ptr();
        let len = buf.len();

        let sqe = io_uring::opcode::Read::new(types::Fd(file.as_raw_fd()), ptr, len as u32)
            .offset(offset as u64)
            .build()
            .user_data(i as u64);
        unsafe {
            uring.submission().push(&sqe).unwrap();
        }

        bufs.push(buf);
    }

    uring.submit()?;
    let mut cqes = uring.completion();
    cqes.sync();

    let mut count = 0;
    for _ in cqes {
        count += 1;
    }

    assert_eq!(count, BLOCK_COUNT);
    Ok(())
}

fn bench_std_read_big_file(c: &mut Criterion) {
    c.bench_function("std_read_big_file", |b| b.iter(std_read_big_file));
}

fn bench_uring_read_big_file(c: &mut Criterion) {
    c.bench_function("uring_read_big_file", |b| b.iter(uring_read_big_file));
}

criterion_group!(benches, bench_std_read_big_file, bench_uring_read_big_file);
criterion_main!(benches);
