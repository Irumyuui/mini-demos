use std::{
    collections::BTreeMap,
    fmt::Debug,
    mem::transmute,
    ops::Bound,
    sync::{atomic::AtomicBool, Arc, Mutex, RwLock},
};

use anyhow::Result;
use bytes::Bytes;

#[derive(Clone)]
struct Key(Bytes, u64);

impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Key")
            .field("str", &self.0)
            .field("seq", &self.1)
            .finish()
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        // rev
        other.1.partial_cmp(&self.1)
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).then(self.1.cmp(&other.1).reverse())
    }
}

impl Eq for Key {}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

pub trait Engine: Send + Sync {}

type Version = u64;

pub struct Watermark {
    readers: BTreeMap<Version, usize>,
}

impl Watermark {
    pub fn new() -> Self {
        Self {
            readers: BTreeMap::new(),
        }
    }

    pub fn add_reader(&mut self, ts: Version) {
        *self.readers.entry(ts).or_default() += 1;
    }

    pub fn remove_reader(&mut self, ts: Version) {
        if let Some(count) = self.readers.get_mut(&ts) {
            *count -= 1;
            if *count == 0 {
                self.readers.remove(&ts);
            }
        }
    }

    pub fn watermark(&self) -> Option<Version> {
        self.readers.first_key_value().map(|(ts, _)| *ts)
    }
}

pub struct Transaction {
    read_ts: Version,
    db: Arc<DBInner>,
    local_data: Arc<RwLock<BTreeMap<Bytes, Bytes>>>,
    committed: Arc<AtomicBool>,
}

impl Transaction {
    fn check_commit(&self) -> Result<()> {
        if self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Transaction already committed"));
        }
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Bytes>> {
        self.check_commit()?;

        if let Some(value) = self.local_data.read().unwrap().get(key) {
            if value.is_empty() {
                return Ok(None);
            }
            return Ok(Some(value.clone()));
        }

        self.db.get_with_ts(key, self.read_ts)
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.check_commit()?;
        let key = Bytes::copy_from_slice(key);
        let value = Bytes::copy_from_slice(value);
        self.local_data.write().unwrap().insert(key, value);
        Ok(())
    }

    pub fn delete(&self, key: &[u8]) -> Result<()> {
        self.put(key, b"")
    }

    pub fn commit(self: Arc<Self>) -> Result<()> {
        self.committed
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .map_err(|_| anyhow::anyhow!("Transaction already committed"))?;

        let _commit_lock = self.db.mvcc().comit_lock.lock().unwrap();
        let ts = self.db.write_batch_inner(
            self.local_data
                .read()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref())),
        )?;
        println!("[Info] Commit ts: {}", ts);
        Ok(())
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        self.db
            .mvcc()
            .ts
            .lock()
            .unwrap()
            .watermark
            .remove_reader(self.read_ts);
    }
}

struct MvccVersionRecord {
    pub last_commit_ts: Version,
    pub watermark: Watermark,
}

pub struct Mvcc {
    write_lock: Mutex<()>,
    comit_lock: Mutex<()>,
    ts: Arc<Mutex<MvccVersionRecord>>,
}

impl Mvcc {
    pub fn new(init_ts: Version) -> Self {
        Self {
            write_lock: Mutex::new(()),
            comit_lock: Mutex::new(()),
            ts: Arc::new(Mutex::new(MvccVersionRecord {
                last_commit_ts: init_ts,
                watermark: Watermark::new(),
            })),
        }
    }

    pub fn last_commit_ts(&self) -> Version {
        self.ts.lock().unwrap().last_commit_ts
    }

    pub fn update_commit_ts(&self, ts: Version) {
        self.ts.lock().unwrap().last_commit_ts = ts
    }

    pub fn watermark(&self) -> Version {
        let ts = self.ts.lock().unwrap();
        ts.watermark.watermark().unwrap_or(ts.last_commit_ts)
    }

    pub fn new_txn(&self, inner: Arc<DBInner>) -> Arc<Transaction> {
        let mut ts = self.ts.lock().unwrap();
        let read_ts = ts.last_commit_ts;
        ts.watermark.add_reader(read_ts);

        Arc::new(Transaction {
            read_ts,
            db: inner,
            local_data: Default::default(),
            committed: Default::default(),
        })
    }
}

pub struct DBInner {
    data: Arc<RwLock<BTreeMap<Key, Bytes>>>,
    mvcc: Mvcc,
}

impl DBInner {
    fn open() -> Result<Self> {
        Ok(Self {
            data: Arc::new(RwLock::new(BTreeMap::new())),
            mvcc: Mvcc::new(0), // from wal
        })
    }

    fn mvcc(&self) -> &Mvcc {
        &self.mvcc
    }

    pub fn get(self: &Arc<Self>, key: &[u8]) -> Result<Option<Bytes>> {
        self.mvcc().new_txn(self.clone()).get(key)
    }

    fn get_with_ts(&self, key: &[u8], ts: Version) -> Result<Option<Bytes>> {
        let search_key = Key(Bytes::from_static(unsafe { transmute(key) }), ts);

        if let Some(value) = self
            .data
            .read()
            .unwrap()
            .range((Bound::Included(&search_key), Bound::Unbounded))
            .next()
        {
            return Ok(Some(value.1.clone()));
        }
        Ok(None)
    }

    pub fn put(self: &Arc<Self>, key: &[u8], value: &[u8]) -> Result<()> {
        self.write_batch(std::iter::once((key, value)))
    }

    pub fn delete(self: &Arc<Self>, key: &[u8]) -> Result<()> {
        self.write_batch(std::iter::once((key, [].as_ref())))
    }

    pub fn write_batch<'a>(
        self: &'a Arc<Self>,
        batch: impl Iterator<Item = (&'a [u8], &'a [u8])>,
    ) -> Result<()> {
        let txn = self.mvcc().new_txn(self.clone());
        for (k, v) in batch {
            txn.put(k, v)?;
        }
        txn.commit()?;
        Ok(())
    }

    fn write_batch_inner<'a>(
        &'a self,
        batch: impl Iterator<Item = (&'a [u8], &'a [u8])>,
    ) -> Result<Version> {
        let _write_lock = self.mvcc.write_lock.lock().unwrap();
        let ts = self.mvcc().last_commit_ts() + 1;
        for (k, v) in batch {
            let key = Key(Bytes::copy_from_slice(k), ts);
            let value = Bytes::copy_from_slice(v);
            self.data.write().unwrap().insert(key, value);
        }
        self.mvcc().update_commit_ts(ts);
        Ok(ts)
    }
}

pub struct DB {
    inner: Arc<DBInner>,
}

impl DB {
    pub fn open() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(DBInner::open()?),
        })
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Bytes>> {
        self.inner.get(key)
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.inner.put(key, value)
    }

    pub fn delete(&self, key: &[u8]) -> Result<()> {
        self.inner.delete(key)
    }

    pub fn write_batch<'a>(
        &'a self,
        batch: impl Iterator<Item = (&'a [u8], &'a [u8])>,
    ) -> Result<()> {
        self.inner.write_batch(batch)
    }

    pub fn new_txn(&self) -> Arc<Transaction> {
        self.inner.mvcc().new_txn(self.inner.clone())
    }
}

fn main() -> anyhow::Result<()> {
    let db = DB::open()?;

    let txn = db.new_txn();
    db.put(b"key1", b"value1")?;
    assert_eq!(None, txn.get(b"key1").unwrap());
    assert_eq!(
        Some(Bytes::from_static(b"value1")),
        db.get(b"key1").unwrap()
    );

    txn.put(b"key1", b"value2")?;
    assert_eq!(
        Some(Bytes::from_static(b"value2")),
        txn.get(b"key1").unwrap()
    );

    txn.commit()?;
    assert_eq!(
        Some(Bytes::from_static(b"value2")),
        db.get(b"key1").unwrap()
    );

    println!("{:?}", db.get(b"key1"));

    Ok(())
}
