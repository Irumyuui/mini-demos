use std::{
    cell::RefCell,
    future::poll_fn,
    os::fd::{AsRawFd, FromRawFd, RawFd},
    pin::Pin,
    rc::Rc,
    task::{Poll, Waker, ready},
};

use crate::uring::op::Op;

use super::close::Close;

#[derive(Clone)]
pub(crate) struct SharedFd {
    inner: Rc<Inner>,
}

struct Inner {
    fd: RawFd,
    state: RefCell<State>,
}

enum State {
    Init,

    Waiting(Option<Waker>),

    Closing(Op<Close>),

    Closed,
}

impl SharedFd {
    pub(crate) fn new<F: AsRawFd>(fd: F) -> Self {
        Self {
            inner: Rc::new(Inner {
                fd: fd.as_raw_fd(),
                state: RefCell::new(State::Init),
            }),
        }
    }

    pub(crate) fn raw_fd(&self) -> RawFd {
        self.inner.fd
    }

    pub(crate) async fn close(mut self) {
        if let Some(inner) = Rc::get_mut(&mut self.inner) {
            inner.submit_close_op();
        }
        self.inner.closed().await;
    }
}

impl Inner {
    fn submit_close_op(&mut self) {
        let state = RefCell::get_mut(&mut self.state);

        *state = match Op::close(self.fd) {
            Ok(op) => State::Closing(op),
            Err(_) => {
                let _ = unsafe { std::fs::File::from_raw_fd(self.fd) };
                State::Closed
            }
        }
    }

    async fn closed(&self) {
        poll_fn(|cx| {
            let mut state = self.state.borrow_mut();

            match &mut *state {
                State::Init => {
                    *state = State::Waiting(Some(cx.waker().clone()));
                    Poll::Pending
                }
                State::Waiting(Some(w)) => {
                    if !w.will_wake(cx.waker()) {
                        *w = cx.waker().clone();
                    }
                    Poll::Pending
                }

                State::Waiting(None) => {
                    *state = State::Waiting(Some(cx.waker().clone()));
                    Poll::Pending
                }

                State::Closing(op) => {
                    let _ = ready!(Pin::new(op).poll(cx));
                    *state = State::Closed;
                    Poll::Ready(())
                }

                State::Closed => Poll::Ready(()),
            }
        })
        .await;
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let State::Init | State::Waiting(..) = RefCell::get_mut(&mut self.state) {
            self.submit_close_op();
        }
    }
}
