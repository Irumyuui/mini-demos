use std::os::fd::AsRawFd;

const CHUNK_SIZE: usize = 1024;
const QUEUE_DEPTH: u32 = 1024;

fn main() -> anyhow::Result<()> {
    // Register io_uring...
    let mut ring = io_uring::IoUring::new(QUEUE_DEPTH)?;

    let file = std::fs::File::open("io-uring.md")?;
    let file_size = file.metadata()?.len();

    // Init buffers.
    let chunks = (file_size + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64;
    let mut bufs: Vec<Vec<u8>> = Vec::with_capacity(chunks as _);
    for _ in 0..chunks {
        bufs.push(vec![0; CHUNK_SIZE]);
    }

    for (i, buf) in bufs.iter_mut().enumerate() {
        let offset = i * CHUNK_SIZE;
        let read_entry = io_uring::opcode::Read::new(
            io_uring::types::Fd(file.as_raw_fd()),
            buf.as_mut_ptr(),
            buf.len() as _,
        )
        .offset(offset as _)
        .build()
        .user_data(i as _);

        unsafe {
            ring.submission().push(&read_entry)?;
        }
    }

    // Will block until read finished.
    // ring.submit_and_wait(chunks as _)?;

    // let mut data = Vec::with_capacity(file_size as _);
    // for _ in 0..chunks {
    //     if let Some(cqe) = ring.completion().next() {
    //         let chunk_index = cqe.user_data();
    //         let bytes_read = cqe.result() as _;

    //         if bytes_read > 0 {
    //             data.extend_from_slice(&bufs[chunk_index as usize][..bytes_read]);
    //         } else {
    //             anyhow::bail!("read error");
    //         }
    //     }
    // }

    // Just try.
    ring.submit()?;

    let mut data = Vec::with_capacity(file_size as _);
    let mut completed = 0;
    while completed < chunks {
        ring.submit_and_wait(1)?;

        while let Some(cqe) = ring.completion().next() {
            completed += 1;

            let index = cqe.user_data() as usize;
            let bytes_read = cqe.result();
            anyhow::ensure!(bytes_read >= 0, "read error");

            data.extend_from_slice(&bufs[index][..bytes_read as _]);
        }
    }

    let data = String::from_utf8(data)?;
    println!("Read data: ");
    println!("{}", data);

    Ok(())
}
