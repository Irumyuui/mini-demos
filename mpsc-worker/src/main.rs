use std::{
    collections::HashMap,
    mem::transmute,
    sync::{atomic::AtomicBool, Arc, RwLock},
    thread::JoinHandle,
};

use anyhow::{Context, Result};
use bytes::Bytes;
use crossbeam::channel::{Receiver, Sender};

enum WriteTask {
    Put(Bytes, Bytes, Sender<()>),
    Delete(Bytes, Sender<()>),
}

struct MiniWorkerImpl {
    core: Arc<RwLock<HashMap<Bytes, Bytes>>>,
    task_sender: Sender<WriteTask>,
    close_sender: Sender<()>,
    closed: AtomicBool,
}

impl MiniWorkerImpl {
    pub fn new(task_sender: Sender<WriteTask>, close_sender: Sender<()>) -> Self {
        Self {
            core: Arc::new(RwLock::new(HashMap::new())),
            task_sender,
            close_sender,
            closed: AtomicBool::new(false),
        }
    }

    fn task_result_channel() -> (Sender<()>, Receiver<()>) {
        crossbeam::channel::bounded(1)
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Bytes>> {
        self.check_closed()?;
        let search_key = Bytes::from_static(unsafe { transmute(key) });
        let data = self.core.read().unwrap().get(&search_key).cloned();
        Ok(data)
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.check_closed()?;
        let (s, t) = Self::task_result_channel();
        let task = WriteTask::Put(
            Bytes::copy_from_slice(key),
            Bytes::copy_from_slice(value),
            s,
        );
        self.task_sender.send(task).context("Closed")?;
        t.recv().context("Closed")?;
        Ok(())
    }

    pub fn delete(&self, key: &[u8]) -> Result<()> {
        self.check_closed()?;
        let (s, t) = Self::task_result_channel();
        let task = WriteTask::Delete(Bytes::copy_from_slice(key), s);
        self.task_sender.send(task).context("Closed")?;
        t.recv().context("Closed")?;
        Ok(())
    }

    pub fn do_write(&self, close_sign: Receiver<()>, task_recv: Receiver<WriteTask>) -> Result<()> {
        loop {
            crossbeam::select! {
                recv(task_recv) -> task => {
                    self.do_write_task(task?)?;
                }
                recv(close_sign) -> _ => {
                    tracing::info!("[Close] Worker is closing.");
                    return Ok(())
                }
            }
        }
    }

    fn do_write_task(&self, task: WriteTask) -> Result<()> {
        match task {
            WriteTask::Put(key, value, s) => {
                self.core.write().unwrap().insert(key, value);
                s.send(()).context("result error")?;
            }
            WriteTask::Delete(key, s) => {
                self.core.write().unwrap().remove(&key);
                s.send(()).context("result error")?;
            }
        }
        Ok(())
    }

    pub fn close(&self) -> Result<()> {
        self.check_closed()?;
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
        self.close_sender.send(()).context("Closed")?;
        Ok(())
    }

    fn check_closed(&self) -> Result<()> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Worker is closed"));
        }
        Ok(())
    }
}

impl Drop for MiniWorkerImpl {
    fn drop(&mut self) {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        tracing::warn!("Worker is not closed, try to close it.");
        self.close().unwrap();
    }
}

#[derive(Clone)]
pub struct MiniWorker {
    inner: Arc<MiniWorkerImpl>,
    write_thread: Arc<Option<JoinHandle<()>>>,
    closed: Arc<AtomicBool>,
}

impl MiniWorker {
    pub fn new() -> Self {
        let (task_sender, task_recv) = crossbeam::channel::unbounded();
        let (close_sender, close_recv) = crossbeam::channel::bounded(1);

        let inner = Arc::new(MiniWorkerImpl::new(task_sender, close_sender));

        let core = inner.clone();
        let write_thread = std::thread::spawn(move || {
            core.do_write(close_recv, task_recv).unwrap();
        });

        Self {
            inner,
            write_thread: Arc::new(Some(write_thread)),
            closed: Arc::new(AtomicBool::new(false)),
        }
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

    pub fn close(&mut self) -> Result<()> {
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
        self.inner.close()?;
        let thread = std::mem::replace(&mut self.write_thread, Arc::new(None));
        Arc::into_inner(thread).flatten().unwrap().join().unwrap();
        Ok(())
    }
}

impl Drop for MiniWorker {
    fn drop(&mut self) {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        tracing::warn!("Worker is not closed, try to close it.");
        self.close().unwrap();
    }
}

fn main() {
    let worker = MiniWorker::new();
    worker.put(b"hello", b"world").unwrap();
    let data = worker.get(b"hello").unwrap();
    println!("{:?}", data);
}
