use std::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use rustix::thread::CpuSet;

pub mod slab;

/// Pin the current thread to a seleted core.
///
/// ```rust
/// use zahorikv::worker::pin_thread_on;
///
/// std::thread::spawn(|| {
///     // Get the current core id.
///     let cpuid = rustix::thread::sched_getcpu();
///     pin_thread_on(cpuid).expect("pin thread failed");
/// });
/// ```
#[tracing::instrument]
pub fn pin_thread_on(core_id: usize) -> std::io::Result<()> {
    let mut cpuset = CpuSet::new();
    cpuset.set(core_id);
    rustix::thread::sched_setaffinity(None, &cpuset)?;
    Ok(())
}

pub fn to_die<E: std::error::Error>(err: E) -> ! {
    tracing::error!("fatal error: {}", err);
    std::process::exit(1)
}

// pub struct InplaceVec<const N: usize, T> {
//     data: [MaybeUninit<T>; N],
//     len: usize,
// }

// impl<const N: usize, T> InplaceVec<N, T> {
//     pub fn new() -> Self {
//         Self {
//             data: unsafe { MaybeUninit::uninit() },
//             len: 0,
//         }
//     }

//     pub fn try_push(&mut self, value: T) -> Result<(), T> {
//         if self.len < N {
//             unsafe {
//                 self.data[self.len].as_mut_ptr().write(value);
//             }
//             self.len += 1;
//             Ok(())
//         } else {
//             Err(value)
//         }
//     }

//     pub fn pop(&mut self) -> Option<T> {
//         if self.len > 0 {
//             self.len -= 1;
//             unsafe { Some(self.data[self.len].assume_init_read()) }
//         } else {
//             None
//         }
//     }

//     pub fn len(&self) -> usize {
//         self.len
//     }

//     pub fn is_empty(&self) -> bool {
//         self.len == 0
//     }
// }

// impl<const N: usize, T> Drop for InplaceVec<N, T> {
//     fn drop(&mut self) {
//         for i in 0..self.len {
//             unsafe {
//                 self.data[i].assume_init_drop();
//             }
//         }
//     }
// }

// impl<const N: usize, T> Index<usize> for InplaceVec<N, T> {
//     type Output = T;

//     fn index(&self, index: usize) -> &Self::Output {
//         if index < self.len {
//             unsafe { &*self.data[index].as_ptr() }
//         } else {
//             panic!("index out of bounds")
//         }
//     }
// }

// impl<const N: usize, T> IndexMut<usize> for InplaceVec<N, T> {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         if index < self.len {
//             unsafe { &mut *self.data[index].as_mut_ptr() }
//         } else {
//             panic!("index out of bounds")
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use std::{
//         cell::RefCell,
//         rc::Rc,
//         sync::atomic::{AtomicUsize, Ordering},
//     };

//     use crate::utils::InplaceVec;

//     #[test]
//     fn test_inplace_vec() {
//         let mut vec = InplaceVec::<4, i32>::new();
//         assert!(vec.is_empty());
//         assert_eq!(vec.len(), 0);

//         vec.try_push(1).unwrap();
//         assert!(!vec.is_empty());
//         assert_eq!(vec.len(), 1);

//         vec.try_push(2).unwrap();
//         assert_eq!(vec.len(), 2);

//         assert_eq!(vec.pop(), Some(2));
//         assert_eq!(vec.len(), 1);
//     }

//     #[test]
//     fn test_inplace_vec_drop() {
//         static COUNT: AtomicUsize = AtomicUsize::new(0);

//         struct DropCounter;

//         impl Drop for DropCounter {
//             fn drop(&mut self) {
//                 COUNT.fetch_add(1, Ordering::SeqCst);
//             }
//         }
//         impl DropCounter {
//             fn new() -> Self {
//                 COUNT.fetch_sub(1, Ordering::SeqCst);
//                 Self
//             }
//         }

//         let mut vec = InplaceVec::<10, DropCounter>::new();
//         assert_eq!(COUNT.load(Ordering::SeqCst), 0);
//         for _ in 0..10 {
//             let _ = vec.try_push(DropCounter::new());
//         }
//         assert_eq!(COUNT.load(Ordering::SeqCst), 10);

//         drop(vec);
//         assert_eq!(COUNT.load(Ordering::SeqCst), 0);
//     }
// }
