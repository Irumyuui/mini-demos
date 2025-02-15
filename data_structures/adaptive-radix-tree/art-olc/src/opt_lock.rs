use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Debug)]
pub enum OptLockError {
    VersionMismatch,
    Locked,
    Obsoleted,
}

use OptLockError::*;

type OptResult<T> = Result<T, OptLockError>;

#[derive(Debug)]
pub struct OptLock<T> {
    inner: UnsafeCell<T>,

    // version: 62 bit | lock: 1 bit | obsolete: 1 bit |
    version: AtomicU64,
}

// OptLock is thread safe.
// If the `T` is `Send`, then means `OptLock<T>` is `Send`.
// If the `T` is `Send + Sync`, then means `OptLock<T>` is `Sync`.
unsafe impl<T: Send> Send for OptLock<T> {}
unsafe impl<T: Send + Sync> Sync for OptLock<T> {}

impl<T> OptLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: UnsafeCell::new(data),
            version: AtomicU64::new(0),
        }
    }

    /// Read and restart.
    ///
    /// # Example
    ///
    /// ```
    /// use opt_lock::OptLock;
    ///
    /// fn main() {
    ///     let lock = OptLock::from(0);
    ///     
    ///     'retry: loop {
    ///         match lock.read() {
    ///             Ok(guard) => {
    ///                 // Do something with guard...
    ///                 // Maybe just puls 1.
    ///                 println!("{}", *guard + 1);
    ///                 // And remember `check_version` on every action after.
    ///                 match guard.check_version() {
    ///                     Ok(_) => break 'retry,
    ///                     Err(_) => continue 'retry,
    ///                 }
    ///             }
    ///             Err(_) => {
    ///                 // The data is obsoleted or out of version...
    ///                 // So need restart it.
    ///                 continue 'retry;
    ///             },
    ///         }
    ///     }
    /// }
    /// ```
    pub fn read(&self) -> OptResult<OptReadGuard<'_, T>> {
        let version = self.check_version()?;
        Ok(OptReadGuard {
            lock: self,
            locked_version: version,
        })
    }

    /// Ready to write
    ///
    /// # Example
    ///
    /// ```
    /// use opt_lock::OptLock;
    ///
    /// fn main() {
    ///     let lock = OptLock::from(0);
    ///
    ///     'retry: loop {
    ///         match lock.write() {
    ///             Ok(mut write_guard) => {
    ///                 // Do something with write_guard...
    ///                 *write_guard += 1;
    ///                 // And remember `write_guard` will be droped after the block.
    ///                 break 'retry;
    ///             }
    ///             Err(_) => continue 'retry,
    ///         }
    ///     }
    /// }
    /// ```
    pub fn write(&self) -> OptResult<OptWriteGuard<'_, T>> {
        let version = self.check_version()?;

        match self.version.compare_exchange(
            version,
            Self::mark_lock(version),
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Ok(OptWriteGuard { lock: self }),
            Err(_) => Err(VersionMismatch),
        }
    }

    fn mark_lock(version: u64) -> u64 {
        version + 0b10
    }

    pub fn mark_obsolte(self) {
        self.version.fetch_or(0b01, Ordering::Release);
    }

    fn is_obsolted(version: u64) -> bool {
        version & 0b01 != 0
    }

    fn is_locked(version: u64) -> bool {
        version & 0b10 != 0
    }

    fn check_version(&self) -> OptResult<u64> {
        let version = self.version.load(Ordering::Acquire);
        if Self::is_obsolted(version) {
            return Err(Obsoleted);
        }
        if Self::is_locked(version) {
            return Err(Locked);
        }
        Ok(version)
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T> From<T> for OptLock<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Debug)]
pub struct OptReadGuard<'a, T: 'a> {
    lock: &'a OptLock<T>,
    locked_version: u64,
}

impl<T> OptReadGuard<'_, T> {
    pub fn check_version(self) -> OptResult<()> {
        if self.locked_version == self.lock.check_version()? {
            drop(self);
            Ok(())
        } else {
            Err(VersionMismatch)
        }
    }
}

impl<T> Deref for OptReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.lock.inner.get().as_ref().unwrap() }
    }
}

#[derive(Debug)]
pub struct OptWriteGuard<'a, T: 'a> {
    lock: &'a OptLock<T>,
}

impl<T> Deref for OptWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.lock.inner.get().as_ref().unwrap() }
    }
}

impl<T> DerefMut for OptWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.lock.inner.get().as_mut().unwrap() }
    }
}

impl<T> Drop for OptWriteGuard<'_, T> {
    // When the write_guard is droped, the lock will be released,
    // and the version will be increased.
    fn drop(&mut self) {
        self.lock.version.fetch_add(0b10, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use crate::opt_lock::OptLock;

    #[test]
    fn multi_threads() {
        const ONE_LOOP: usize = 100000;
        const THREADS: usize = 10;
        const RESULT: usize = ONE_LOOP * THREADS;

        let raw_lock = Arc::new(OptLock::from(0));

        let threads = (0..THREADS)
            .map(|_| {
                let lock = raw_lock.clone();

                thread::spawn(move || {
                    for _ in 0..ONE_LOOP {
                        'retry: loop {
                            match lock.write() {
                                Ok(mut write_guard) => {
                                    *write_guard += 1;
                                    break 'retry;
                                }
                                Err(_) => continue 'retry,
                            }
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        for th in threads.into_iter() {
            th.join().unwrap();
        }

        let read_guard = raw_lock.read().unwrap();
        assert_eq!(*read_guard, RESULT);
        read_guard.check_version().unwrap();
    }

    #[test]
    fn wait_release_write_lock() {
        let lock = OptLock::from(0);
        let w = lock.write().unwrap();
        let _r = lock.read().unwrap_err();
        drop(w);

        let r = lock.read().unwrap();
        assert_eq!(*r, 0);
    }
}
