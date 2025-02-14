use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

struct Mgr {
    state_lock: Mutex<()>,
    inner: RwLock<Arc<i32>>,
}

impl Mgr {
    pub fn new(n: i32) -> Self {
        Self {
            state_lock: Mutex::new(()),
            inner: RwLock::new(Arc::new(n)),
        }
    }

    pub fn read(&self) -> Arc<i32> {
        let _guard = self.state_lock.lock().unwrap();
        self.inner.read().unwrap().clone()
    }

    pub fn update(&self, n: i32) {
        let _snapshot = { self.read() };

        let _lock = self.state_lock.lock().unwrap();
        let mut guard = self.inner.write().unwrap();
        *guard = Arc::new(n);
    }
}

fn main() {
    let mgr = Arc::new(Mgr::new(1));

    let th1 = {
        let mgr = mgr.clone();
        std::thread::spawn(move || {
            for _ in 0..60 {
                let data = mgr.read();
                println!("th1: {}", data);
                std::thread::sleep(Duration::from_millis(200));
            }
        })
    };

    let th2 = {
        let mgr = mgr.clone();
        std::thread::spawn(move || {
            for i in 0..30 {
                mgr.update(i);
                std::thread::sleep(Duration::from_millis(400));
            }
        })
    };

    th1.join().unwrap();
    th2.join().unwrap();
}
