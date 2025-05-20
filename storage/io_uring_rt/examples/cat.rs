use std::path::PathBuf;

use clap::Parser;
use uring_rt::uring::rt::default_rt;

#[derive(Debug, clap::Parser)]
#[command(name = "cat", about = "io_uring cat")]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let arg = Args::parse();
    let path = arg.path;
    if !path.exists() {
        eprintln!("File does not exist: {:?}", path);
        std::process::exit(1);
    }

    let main_async = async || -> anyhow::Result<()> {
        let file = uring_rt::uring::fs::File::open(&path).await?;
        let file_size = file.metadata().await?.size();

        let (res, buf) = file.read_at(vec![0_u8; file_size as usize], 0).await;
        let read = res?;
        assert_eq!(read, file_size as usize);

        let content = String::from_utf8(buf)?;
        println!("{}", content);

        Ok(())
    };

    default_rt()?.block_on(async move { main_async().await })
}
