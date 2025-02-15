#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("gen_file/write_empty.txt")?;

    let ring = rio::new()?;

    let data = b"hello world";
    ring.write_at(&file, data, 4096 * 1024 * 1024).await?;

    Ok(())
}
