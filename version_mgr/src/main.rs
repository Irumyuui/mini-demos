use std::{
    collections::VecDeque,
    sync::{atomic::AtomicUsize, Arc, RwLock},
};

pub struct Versions<T> {
    queue: RwLock<VecDeque<Arc<T>>>,
    seq: AtomicUsize,
}

impl<T> Versions<T> {
    pub fn new(begin_seq: usize) -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            seq: AtomicUsize::new(begin_seq),
        }
    }

    pub fn push(&self, item: Arc<T>) {
        self.queue.write().unwrap().push_back(item);
        self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn current(&self) -> Option<Arc<T>> {
        self.queue.read().unwrap().back().cloned()
    }

    pub fn pop_deleted_versions(&self) -> Vec<Arc<T>> {
        let mut guard = self.queue.write().unwrap();
        let mut deleted_versions = Vec::new();
        // 从队列头开始寻找，如果找到 Arc<T> 的引用计数为 1，说明没有其他引用，可以删除
        while !guard.is_empty() {
            if Arc::strong_count(guard.front().unwrap()) == 1 {
                deleted_versions.push(guard.pop_front().unwrap());
            } else {
                break;
            }
        }
        deleted_versions
    }

    pub fn current_seq(&self) -> usize {
        self.seq.load(std::sync::atomic::Ordering::SeqCst)
    }
}

fn main() {
    let ref_items = (0..10).map(|i| Arc::new(i)).collect::<Vec<_>>();
    let versions = Versions::new(0);
    for item in ref_items.iter() {
        versions.push(item.clone());
    }

    let current = versions.current().unwrap();
    println!("current: {}", current);

    drop(ref_items);
    let deleted_versions = versions.pop_deleted_versions();
    println!("deleted versions: {:?}", deleted_versions);

    drop(current);
    let deleted_versions = versions.pop_deleted_versions();
    println!("deleted versions: {:?}", deleted_versions);
}
