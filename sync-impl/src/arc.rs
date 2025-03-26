use std::{
    cell::UnsafeCell,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering::*},
};

macro_rules! acquire {
    ($e: expr) => {
        std::sync::atomic::fence(Acquire);
    };
}

#[derive(Debug)]
pub struct ArcInner<T> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    data: UnsafeCell<ManuallyDrop<T>>,
}

impl<T> ArcInner<T> {
    fn new(data: T) -> Self {
        Self {
            strong: AtomicUsize::new(1),
            weak: AtomicUsize::new(1),
            data: UnsafeCell::new(ManuallyDrop::new(data)),
        }
    }
}

#[derive(Debug)]
pub struct Arc<T> {
    inner: NonNull<ArcInner<T>>,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        self.inner().strong.fetch_add(1, Relaxed);
        Self { inner: self.inner }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner().data.get() }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.inner().strong.fetch_sub(1, Release) != 1 {
            return;
        }
        acquire!(self.inner().strong);
        let _weak = Weak { inner: self.inner };
        unsafe { ManuallyDrop::drop(&mut *self.inner().data.get()) };
    }
}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: NonNull::from(Box::leak(Box::new(ArcInner::new(data)))),
        }
    }

    fn inner(&self) -> &ArcInner<T> {
        unsafe { self.inner.as_ref() }
    }

    pub fn downgrade(this: &Self) -> Weak<T> {
        let mut weak_count = this.inner().weak.load(Relaxed);
        loop {
            if weak_count == usize::MAX {
                std::hint::spin_loop();
                weak_count = this.inner().weak.load(Relaxed);
                continue;
            }

            match this
                .inner()
                .weak
                .compare_exchange(weak_count, weak_count + 1, Acquire, Relaxed)
            {
                Ok(_) => return Weak { inner: this.inner },
                Err(current_count) => weak_count = current_count,
            }
        }
    }

    fn is_unique(&mut self) -> bool {
        match self
            .inner()
            .weak
            .compare_exchange(1, usize::MAX, Acquire, Relaxed)
        {
            Ok(_) => {
                let is_unique = self.inner().strong.load(Acquire) == 1;
                self.inner().weak.store(1, Release);
                is_unique
            }
            Err(_) => false,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_unique() {
            Some(unsafe { &mut *self.inner().data.get() })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Weak<T> {
    inner: NonNull<ArcInner<T>>,
}

unsafe impl<T: Send + Sync> Send for Weak<T> {}
unsafe impl<T: Send + Sync> Sync for Weak<T> {}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.inner().weak.fetch_sub(1, Release) == 1 {
            acquire!(self.inner().weak);
            let _inner = unsafe { Box::from_raw(self.inner.as_ptr()) };
        }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        self.inner().weak.fetch_add(1, Relaxed);
        Self { inner: self.inner }
    }
}

impl<T> Weak<T> {
    fn inner(&self) -> &ArcInner<T> {
        unsafe { self.inner.as_ref() }
    }

    pub fn upgrade(this: &Weak<T>) -> Option<Arc<T>> {
        let mut strong_count = this.inner().strong.load(Acquire);
        loop {
            if strong_count == 0 {
                return None;
            }

            match this.inner().strong.compare_exchange(
                strong_count,
                strong_count + 1,
                Acquire,
                Relaxed,
            ) {
                Ok(_) => return Some(Arc { inner: this.inner }),
                Err(current_count) => strong_count = current_count,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    #[test]
    fn test_arc_basic() {
        let arc = Arc::new(42);
        assert_eq!(*arc, 42);

        let cloned = arc.clone();
        assert_eq!(*cloned, 42);

        assert_eq!(arc.deref(), &42);
    }

    #[test]
    fn test_arc_drop() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct TestDropper;

        impl Drop for TestDropper {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        {
            let arc = Arc::new(TestDropper);
            let _cloned = arc.clone();
            assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 0);
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_weak_upgrade() {
        let arc = Arc::new(100);
        let weak = Arc::downgrade(&arc);

        let upgraded = Weak::upgrade(&weak).unwrap();
        assert_eq!(*upgraded, 100);

        drop(arc);
        drop(upgraded);
        assert!(Weak::upgrade(&weak).is_none());
    }

    #[test]
    fn test_weak_cloning() {
        let arc = Arc::new(());
        let weak1 = Arc::downgrade(&arc);
        let weak2 = weak1.clone();

        assert!(Weak::upgrade(&weak1).is_some());
        assert!(Weak::upgrade(&weak2).is_some());

        drop(arc);
        assert!(Weak::upgrade(&weak1).is_none());
        assert!(Weak::upgrade(&weak2).is_none());
    }

    #[test]
    fn test_get_mut() {
        let mut arc = Arc::new(5);

        *Arc::get_mut(&mut arc).unwrap() = 10;
        assert_eq!(*arc, 10);

        let _cloned = arc.clone();
        assert!(Arc::get_mut(&mut arc).is_none());

        let weak = Arc::downgrade(&arc);
        assert!(Arc::get_mut(&mut arc).is_none());

        drop(_cloned);
        drop(weak);
        *Arc::get_mut(&mut arc).unwrap() = 20;
        assert_eq!(*arc, 20);
    }

    #[test]
    fn test_concurrent_access() {
        let arc = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let arc_clone = arc.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    arc_clone.fetch_add(1, Ordering::Relaxed);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(arc.load(Ordering::Relaxed), 10000);
    }

    #[test]
    fn test_inner_drop() {
        static INNER_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct InnerTest;

        impl Drop for InnerTest {
            fn drop(&mut self) {
                INNER_DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        let arc = Arc::new(InnerTest);
        let weak = Arc::downgrade(&arc);

        drop(arc);
        assert_eq!(INNER_DROP_COUNT.load(Ordering::SeqCst), 1);

        drop(weak);
        assert_eq!(INNER_DROP_COUNT.load(Ordering::SeqCst), 1);
    }
}
