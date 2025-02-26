use std::{fs::OpenOptions, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let path = PathBuf::from("./test_file");

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(&path)?;
    file.set_len(1024)?;

    let mut mmap_file = unsafe { memmap2::MmapMut::map_mut(&file)? };

    let data = b"hello";
    mmap_file[..data.len()].copy_from_slice(data);
    mmap_file.flush()?;

    let len = file.metadata()?.len();
    println!("File size: {}", len);

    let real_len = data.len();
    println!("Real size: {}", real_len);

    file.set_len(real_len as _)?;
    file.sync_all()?;

    drop(mmap_file);
    drop(file);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(&path)?;
    let mmap_file = unsafe { memmap2::MmapMut::map_mut(&file)? };
    let bytes = String::from_utf8(mmap_file[..].as_ref().to_vec())?;

    println!("Read file: {bytes}");

    Ok(())
}
