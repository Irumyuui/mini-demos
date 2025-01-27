use std::{os::unix::fs::OpenOptionsExt, sync::Arc};

use tokio::{io::AsyncWriteExt, task::JoinSet};

const TOTAL_SIZE: usize = 1024 * 1024 * 1024; // 1GB

const CHUNK_SIZE: usize = 4096 * 1024;

#[repr(align(4096))]
struct AlignedBuffer([u8; CHUNK_SIZE]);

impl Default for AlignedBuffer {
    fn default() -> Self {
        Self([0; CHUNK_SIZE])
    }
}

fn gen_data() -> Vec<u8> {
    let mut data = Vec::with_capacity(TOTAL_SIZE);
    for i in 0..TOTAL_SIZE {
        data.push((i % 127) as u8);
    }
    data
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data = gen_data();

    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("gen_file/data.txt")
        .await?;
    let mut writer = tokio::io::BufWriter::with_capacity(TOTAL_SIZE, file);
    writer.write_all(&data).await?;
    writer.flush().await?;
    drop(writer);

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .custom_flags(nix::libc::O_DIRECT)
        .open("gen_file/data.txt")?;
    let file = Arc::new(file);

    let mut config = rio::Config::default();
    config.depth = 4096;
    let ring = config.start()?;

    let mut bufs = Vec::with_capacity(TOTAL_SIZE / CHUNK_SIZE);

    let mut tasks = JoinSet::new();
    for i in 0..TOTAL_SIZE / CHUNK_SIZE {
        let offset = i * CHUNK_SIZE;
        let file = file.clone();
        let ring = ring.clone();

        tasks.spawn(async move {
            let mut buf = Box::new(AlignedBuffer::default());
            ring.read_at(file.as_ref(), &mut buf.0, offset as u64)
                .await
                .unwrap();
            (i, buf)
        });
    }

    while let Some(res) = tasks.join_next().await {
        bufs.push(res?);
    }

    for (i, _) in bufs.iter() {
        println!("id: {}", i);
    }

    let write_file = std::fs::OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .custom_flags(nix::libc::O_DIRECT)
        .open("gen_file/data2.txt")?;
    let file = Arc::new(write_file);

    let mut tasks = JoinSet::new();

    for (i, buf) in bufs.into_iter() {
        let offset = i * CHUNK_SIZE;
        let file = file.clone();
        let ring = ring.clone();

        tasks.spawn(async move {
            ring.write_at(file.as_ref(), &buf.0, offset as u64)
                .await
                .unwrap();
        });
    }

    tasks.join_all().await;

    Ok(())
}
