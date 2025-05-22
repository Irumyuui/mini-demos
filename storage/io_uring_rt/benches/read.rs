use std::{
    io::{Read, Write},
    path::PathBuf,
};

use criterion::{Criterion, criterion_group, criterion_main};
use uring_rt::uring::rt::default_rt;

fn gen_buffer() -> Vec<u8> {
    let mut buf = vec![0u8; 4 * 1024 * 1024]; // 4MB
    #[allow(clippy::manual_slice_fill)]
    #[allow(clippy::needless_range_loop)]
    for i in 0..buf.len() {
        buf[i] = (i % 256) as u8;
    }
    buf
}

fn bench_block_read(c: &mut Criterion) {
    let path = PathBuf::from("data").join("bench_read");
    scopeguard::defer! {
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    };
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let buf = gen_buffer();
    let mut file = std::fs::File::create(path.clone()).unwrap();
    file.write_all(&buf).unwrap();
    drop(file);

    c.bench_function("block-read", |b| {
        b.iter(|| {
            let mut file = std::fs::File::open(path.clone()).unwrap();
            let mut read_buf = vec![0u8; 4 * 1024 * 1024]; // 4B
            let _ = file.read_exact(&mut read_buf);
        });
    });
}

fn bench_uring_read(c: &mut Criterion) {
    let path = PathBuf::from("data").join("bench_read");
    scopeguard::defer! {
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    };
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let buf = gen_buffer();
    let mut file = std::fs::File::create(path.clone()).unwrap();
    file.write_all(&buf).unwrap();
    drop(file);

    let rt = default_rt().unwrap();

    c.bench_function("uring-read", |b| {
        b.iter(|| {
            rt.block_on(async {
                let file = uring_rt::uring::fs::File::open(path.clone()).await.unwrap();
                let read_buf = vec![0u8; 4 * 1024 * 1024]; // 4MB
                let _ = file.read_at(read_buf, 0).await;
            })
        });
    });
}

criterion_group!(benches, bench_block_read, bench_uring_read);
criterion_main!(benches);
