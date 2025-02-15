use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    os::unix::fs::FileExt,
    path::Path,
    sync::Mutex,
};

use anyhow::{bail, ensure};
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub struct Log {
    file: Mutex<File>,
}

pub struct Entry {
    key: Bytes,
    value: Bytes,
}

pub struct Header {
    active: u8,
    key_len: usize,
    value_len: usize,
}

impl Header {
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(9);
        buf.put_u8(self.active);
        buf.put_u64(self.key_len as u64);
        buf.put_u64(self.value_len as u64);
        buf.freeze()
    }

    pub fn decode(mut buf: &[u8]) -> anyhow::Result<(Self, &[u8])> {
        ensure!(buf.len() >= 9);

        let active = buf.get_u8();
        let key_len = buf.get_u64() as usize;
        let value_len = buf.get_u64() as usize;

        Ok((
            Header {
                active,
                key_len,
                value_len,
            },
            buf,
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pointer {
    len: u32,
    offset: u64,
}

impl Log {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Log {
            file: Mutex::new(file),
        })
    }

    #[cfg(test)]
    fn test_new(file: File) -> Self {
        Self {
            file: Mutex::new(file),
        }
    }

    pub fn write(&self, entries: &[Entry]) -> anyhow::Result<Vec<Pointer>> {
        let mut buf = BytesMut::new();
        let mut ptrs = Vec::with_capacity(entries.len());

        for e in entries {
            let header = Header {
                active: 0,
                key_len: e.key.len(),
                value_len: e.value.len(),
            }
            .encode();

            let ptr = Pointer {
                len: (header.len() + e.key.len()) as _,
                offset: buf.len() as u64,
            };
            ptrs.push(ptr);

            buf.put(header);
            buf.put(e.key.as_ref());
            buf.put(e.value.as_ref());
        }

        let mut file = self.file.lock().unwrap();
        let offset = file.seek(SeekFrom::End(0))?;
        for ptr in &mut ptrs {
            ptr.offset += offset;
        }
        file.write_all(&buf)?;
        Ok(ptrs)
    }

    pub fn read(&self, offset: u64, length: u32, callback: impl Fn(Entry)) -> anyhow::Result<()> {
        let mut buf = vec![0_u8; length as usize];
        {
            let file = self.file.lock().unwrap();
            file.read_exact_at(&mut buf, offset)?;
        }

        let (header, rest) = Header::decode(&buf)?;
        if rest.len() < header.key_len + header.value_len {
            bail!(
                "invalid entry, rest.len: {}, key_len: {}, value_len: {}",
                rest.len(),
                header.key_len,
                header.value_len
            );
        }

        let key = Bytes::copy_from_slice(&rest[..header.key_len]);
        let value =
            Bytes::copy_from_slice(&rest[header.key_len..header.key_len + header.value_len]);

        let entry = Entry { key, value };
        callback(entry);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use bytes::Bytes;
    use tempfile::tempfile;

    #[test]
    fn test_write_and_read() -> anyhow::Result<()> {
        // 创建临时文件
        let temp_file = tempfile()?;
        let log = Log::test_new(temp_file);

        // 创建测试条目
        let entries = vec![
            Entry {
                key: Bytes::from("key1"),
                value: Bytes::from("value1"),
            },
            Entry {
                key: Bytes::from("key2"),
                value: Bytes::from("value2"),
            },
        ];

        // 写入条目
        let ptrs = log.write(&entries)?;
        let entries = Arc::new(entries);

        // 读取并验证条目
        for ptr in ptrs {
            println!("{:?}", ptr);

            let entries = entries.clone();

            log.read(ptr.offset, ptr.len, move |entry| {
                // 根据偏移量和长度确定索引
                let index = ptr.offset as usize / 100; // 示例计算
                assert_eq!(entry.key, entries[index].key);
                assert_eq!(entry.value, entries[index].value);
            })?;
        }

        Ok(())
    }
}
