use std::{
    any::Any,
    cell::RefCell,
    io,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

use futures::future::poll_fn;
use rustix::net::eth::TDLS;
use rustix_uring::{cqueue::Flags, squeue};

use crate::uring::driver::CONTEXT;

use super::driver::Driver;

/// Lifecycle of an operation.
///
/// When a op was been polled,
///
/// ```text
///     Submitted --> Waiting,
///     Waiting   --> Waiting,
///     Completed --> Poll::Ready --> Finished,
///     Ignored   --> panic, because it should not be polled, just await kernal completion.
/// ```
///
/// When a op was been completed,
///
/// ```text
///     Submitted | Waiting --> Completed,  // IO Finished
///     Completed           --> panic, an op has been completed twice.
///     Ignored             --> do nothing, because it was already ignored.
/// ```
#[derive(Debug)]
pub enum Lifecycle {
    Submitted,

    Waiting(Waker),

    Completed(io::Result<u32>, Flags),

    Ignored(Box<dyn Any>),
}

impl Lifecycle {
    // call in poll
    pub fn complete(&mut self, result: io::Result<u32>, flags: Flags) -> bool {
        match std::mem::replace(self, Lifecycle::Submitted) {
            Lifecycle::Submitted => {
                *self = Lifecycle::Completed(result, flags);
                false
            }
            Lifecycle::Waiting(waker) => {
                *self = Lifecycle::Completed(result, flags);
                waker.wake();
                false
            }
            Lifecycle::Completed(..) => unreachable!(),
            Lifecycle::Ignored(..) => true, // finish this op
        }
    }
}

pub struct Op<T: 'static> {
    handle: Weak<RefCell<Driver>>,
    index: usize,
    data: Option<T>,
}

pub struct Completion<T> {
    pub(crate) data: T,
    pub(crate) result: io::Result<u32>,
    pub(crate) flags: Flags,
}

impl<T> Future for Op<T>
where
    T: Unpin + 'static,
{
    type Output = Completion<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let driver = this.handle.upgrade().expect("Driver dropped");
        let mut driver = driver.borrow_mut();

        let lc = driver
            .ops
            .lifecycle
            .get_mut(this.index)
            .expect("Invalid index");

        match std::mem::replace(lc, Lifecycle::Submitted) {
            Lifecycle::Submitted => {
                *lc = Lifecycle::Waiting(cx.waker().clone());
                Poll::Pending
            }
            Lifecycle::Waiting(waker) if !waker.will_wake(cx.waker()) => {
                *lc = Lifecycle::Waiting(cx.waker().clone());
                Poll::Pending
            }
            Lifecycle::Waiting(waker) => {
                *lc = Lifecycle::Waiting(waker);
                Poll::Pending
            }
            Lifecycle::Completed(result, flags) => {
                driver.ops.lifecycle.remove(this.index);
                this.index = usize::MAX;
                Poll::Ready(Completion {
                    data: this.data.take().unwrap(),
                    result,
                    flags,
                })
            }

            // should not poll ignoerd op
            Lifecycle::Ignored(..) => unreachable!(),
        }
    }
}

impl<T> Drop for Op<T> {
    fn drop(&mut self) {
        let driver = self.handle.upgrade().expect("Driver dropped");
        let mut driver = driver.borrow_mut();

        let lc = match driver.ops.lifecycle.get_mut(self.index) {
            Some(lc) => lc,
            None => return, // finished!
        };

        match lc {
            Lifecycle::Submitted | Lifecycle::Waiting(_) => {
                *lc = Lifecycle::Ignored(Box::new(self.data.take()));
            }
            Lifecycle::Completed(..) => {
                driver.ops.lifecycle.remove(self.index);
            }
            Lifecycle::Ignored(..) => unreachable!(),
        }
    }
}

impl<T> Op<T> {
    pub fn new(data: T, driver: &mut Driver, handle: Weak<RefCell<Driver>>) -> Self {
        Op {
            handle,
            index: driver.ops.lifecycle.insert(Lifecycle::Submitted),
            data: Some(data),
        }
    }

    /// Create a new operation and submit it to uring driver.
    ///
    /// Data's ownership is given to driver.
    pub fn submit_with<F>(data: T, f: F) -> io::Result<Self>
    where
        F: FnOnce(&mut T) -> squeue::Entry,
    {
        CONTEXT.with(|cx| {
            let handle = match cx.handle() {
                Some(h) => h,
                None => {
                    return Err(io::Error::other(
                        "Driver not initialized",
                    ));
                }
            };

            let mut driver = handle.borrow_mut();

            if driver.uring.submission().is_full() {
                driver.submit()?;
            }

            let mut op = Op::new(data, &mut driver, Rc::downgrade(&handle));
            let sqe = f(op.data.as_mut().unwrap()).user_data(op.index as u64);
            {
                let mut sq = driver.uring.submission();
                if unsafe { sq.push(&sqe).is_err() } {
                    unreachable!("?");
                }
            }

            driver.submit()?;
            Ok(op)
        })
    }
}

impl<T> Op<T>
where
    T: CompleteAble + Unpin,
{
    pub async fn complete(mut self) -> T::Output {
        poll_fn(move |cx| {
            let comp = std::task::ready!(Pin::new(&mut self).poll(cx));
            let res = CompleteAble::handle_completion(comp);
            Poll::Ready(res)
        })
        .await
    }
}

pub trait CompleteAble: Sized {
    type Output;

    fn handle_completion(comp: Completion<Self>) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use rustix_uring::opcode;

    use super::Op;

    #[test]
    fn test_not_context_submit() {
        let op = Op::submit_with((), |_| opcode::Nop::new().build());
        assert!(op.is_err());
    }
}
