use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::NonNull,
    sync::atomic::{AtomicU64, Ordering},
};

pub enum OptLockError {
    Restart,
}

pub struct OptLock<T> {
    version: AtomicU64,
    data: ManuallyDrop<T>,
}

pub struct ReadGuard<'a, T: 'a> {
    version: u64,
    data: NonNull<OptLock<T>>,
    _marker: PhantomData<&'a OptLock<T>>,
}

pub struct WriteGuard<'a, T: 'a> {
    data: &'a mut OptLock<T>,
}

fn is_obsolete(version: u64) -> bool {
    (version & 1) == 1
}

fn is_locked(version: u64) -> bool {
    (version & 0b10) == 0b10
}

fn set_locked(version: u64) -> u64 {
    version + 2
}

impl<T> OptLock<T> {
    pub fn as_ref(&self) -> &T {
        &self.data
    }

    pub fn as_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn into_data(self) -> T {
        ManuallyDrop::into_inner(self.data)
    }

    pub fn read_lock_or_restart<'a>(&'a self) -> Result<ReadGuard<'a, T>, OptLockError> {
        let mut version = self.version.load(Ordering::Acquire);
        while is_locked(version) {
            if is_obsolete(version) {
                return Err(OptLockError::Restart);
            }
            version = self.version.load(Ordering::Acquire);
        }
        if is_obsolete(version) {
            return Err(OptLockError::Restart);
        }

        let guard = ReadGuard {
            version,
            data: NonNull::from(self),
            _marker: PhantomData,
        };
        Ok(guard)
    }

    pub fn write_lock_or_restart<'a>(&'a self) -> Result<WriteGuard<'a, T>, OptLockError> {
        loop {
            let read_guard = self.read_lock_or_restart()?;
            match read_guard.upgrade_to_write_lock_or_restart() {
                Ok(w) => return Ok(w),
                Err(_) => continue,
            }
        }
    }
}

impl<'a, T: 'a> ReadGuard<'a, T> {
    fn as_ref(&self) -> &OptLock<T> {
        unsafe { self.data.as_ref() }
    }

    pub fn read_unlock_or_restart(&self, version: u64) -> Result<(), OptLockError> {
        let current_version = self.as_ref().version.load(Ordering::Acquire);
        if current_version != version {
            return Err(OptLockError::Restart);
        }
        Ok(())
    }

    pub fn check_or_restart(&self) -> Result<(), OptLockError> {
        self.read_unlock_or_restart(self.version)
    }

    pub fn upgrade_to_write_lock_or_restart(
        self,
    ) -> Result<WriteGuard<'a, T>, (Self, OptLockError)> {
        match self.as_ref().version.compare_exchange(
            self.version,
            set_locked(self.version),
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(_) => Ok(WriteGuard {
                data: unsafe { &mut *self.data.as_ptr() },
            }),
            Err(_) => Err((self, OptLockError::Restart)),
        }
    }
}

impl<'a, T: 'a> WriteGuard<'a, T> {
    pub fn as_ref(&self) -> &OptLock<T> {
        &self.data
    }

    pub fn as_mut(&mut self) -> &mut OptLock<T> {
        &mut self.data
    }

    pub fn write_unlock(self) {
        self.data.version.fetch_add(0b10, Ordering::Release);
    }

    pub fn write_unlock_obsolete(self) {
        self.data.version.fetch_add(0b11, Ordering::Release);
    }
}
