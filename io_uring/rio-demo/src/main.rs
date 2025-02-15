use std::time::Duration;

const CHUNK_SIZE: u64 = 4096 * 256;

#[repr(align(4096))]
struct Aligned([u8; CHUNK_SIZE as usize]);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = rio::Config::default();
    config.print_profile_on_drop = true;
    let ring = config.start().expect("create uring");
    
    // open output file, with `O_DIRECT` set
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("file")
        .expect("open file");

    let out_buf = Box::new(Aligned([42; CHUNK_SIZE as usize]));
    let out_slice: &[u8] = &out_buf.0;

    let mut in_buf = Box::new(Aligned([42; CHUNK_SIZE as usize]));
    let in_slice: &mut [u8] = &mut in_buf.0;

    let mut completions = vec![];

    let pre = std::time::Instant::now();
    for i in 0..(10 * 1024) {
        let at = i * CHUNK_SIZE;

        // By setting the `Link` order,
        // we specify that the following
        // read should happen after this
        // write.
        let write = ring.write_at_ordered(&file, &out_slice, at, rio::Ordering::Link);
        completions.push(write);

        // This operation will not start
        // until the previous linked one
        // finishes.
        let read = ring.read_at(&file, &in_slice, at);
        completions.push(read);
    }

    let post_submit = std::time::Instant::now();

    for completion in completions.into_iter() {
        completion.await?;
    }

    let post_complete = std::time::Instant::now();

    dbg!(post_submit - pre, post_complete - post_submit);

    std::fs::remove_file("file")?;

    Ok(())
}
