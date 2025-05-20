use std::{os::fd::AsRawFd, path::PathBuf, vec};

use clap::Parser;
use io_uring::types;

#[derive(Debug, clap::Parser)]
#[command(name = "cat", about = "io_uring cat")]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

const BLOCK_SIZE: usize = 4096;

fn main() -> anyhow::Result<()> {
    let arg = Args::parse();

    let path = arg.path;
    if !path.exists() {
        eprintln!("File does not exist: {:?}", path);
        std::process::exit(1);
    }

    let file = std::fs::File::open(&path)?;
    let file_size = file.metadata()?.len();

    let mut uring = io_uring::IoUring::new(4096)?;
    let mut buf = vec![0; file_size as usize];

    let entry_count = (file_size as usize).div_ceil(BLOCK_SIZE);
    let mut entries = Vec::with_capacity(entry_count);
    for i in 0..entry_count {
        let offset = i * BLOCK_SIZE;
        let len = if file_size as usize - offset < BLOCK_SIZE {
            file_size as usize - offset
        } else {
            BLOCK_SIZE
        };

        unsafe {
            let ptr = buf.as_mut_ptr().add(offset);
            let entry = io_uring::opcode::Read::new(types::Fd(file.as_raw_fd()), ptr, len as _)
                .offset(offset as _)
                .build()
                .user_data(i as _);
            entries.push(entry);
        }
    }

    unsafe {
        uring.submission().push_multiple(&entries)?;
    }
    uring.submit()?;

    let cqes = uring.completion();
    for e in cqes {
        let _user_data = e.user_data() as usize;
        let res = e.result();
        if res < 0 {
            Err(std::io::Error::from_raw_os_error(-res))?;
        }
    }

    let content = String::from_utf8(buf)?;
    println!("{}", content);

    Ok(())
}
