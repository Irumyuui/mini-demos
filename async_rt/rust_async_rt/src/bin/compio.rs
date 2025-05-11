use compio::{fs::OpenOptions, io::AsyncReadAtExt};

#[compio::main]
async fn main() -> anyhow::Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open("Cargo.toml")
        .await?;

    let len = file.metadata().await?.len() as usize;
    let res = file.read_to_end_at(vec![0; len], 0).await;

    let read = res.0?;
    let buf = res.1;
    assert_eq!(read, len);

    let content = String::from_utf8(buf)?;
    println!("{}", content);

    Ok(())
}
