use std::{
    collections::HashMap,
    fs::OpenOptions,
    os::{
        fd::{AsRawFd, RawFd},
        unix::fs::OpenOptionsExt,
    },
    pin::Pin,
    task::{Context, Poll, Waker},
};

use io_uring::{IoUring, opcode, types};
use itertools::Itertools;
use pin_project::pin_project;

struct IoOpenration {
    waker: Option<Waker>,
    completed: bool,
    result: Option<i32>,
}

pub struct IoUringRuntime {
    ring: IoUring,
    operations: HashMap<u64, IoOpenration>,
    next_token: u64,
}

impl IoUringRuntime {
    pub fn new(entries: u32) -> anyhow::Result<Self> {
        let ring = IoUring::builder().setup_iopoll().build(entries)?;

        // let ring = IoUring::new(entries)?;

        Ok(Self {
            ring,
            operations: HashMap::new(),
            next_token: 0,
        })
    }

    pub fn read_async(&mut self, fd: RawFd, buf: &mut [u8]) -> ReadFuture<'_> {
        let token = self.next_token;
        self.next_token += 1;

        self.operations.insert(
            token,
            IoOpenration {
                waker: None,
                completed: false,
                result: None,
            },
        );

        let read_op = opcode::Read::new(types::Fd(fd), buf.as_mut_ptr() as _, buf.len() as _)
            .build()
            .user_data(token);
        unsafe {
            self.ring.submission().push(&read_op).expect("full queue"); // entires limit
        }
        self.ring.submit().expect("submit failed");

        ReadFuture {
            token,
            runtime: self,
        }
    }

    fn process_complections(&mut self) {
        match self.ring.submit_and_wait(1) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error waiting for completions: {}", e);
            }
        }

        let cqes = self.ring.completion().collect_vec();
        for cqe in cqes {
            let token = cqe.user_data();
            if let Some(op) = self.operations.get_mut(&token) {
                op.completed = true;
                op.result = Some(cqe.result());
                if let Some(waker) = op.waker.take() {
                    waker.wake();
                }
            }
        }
    }
}

#[pin_project]
pub struct ReadFuture<'a> {
    token: u64,
    runtime: &'a mut IoUringRuntime,
}

impl Future for ReadFuture<'_> {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        this.runtime.process_complections();

        if let Some(op) = this.runtime.operations.get_mut(this.token) {
            if op.completed {
                Poll::Ready(op.result.unwrap())
            } else {
                op.waker.replace(cx.waker().clone());
                Poll::Pending
            }
        } else {
            panic!("Invalid op");
        }
    }
}

const CHUNK_SIZE: usize = 4096 * 256;

#[repr(align(4096))]
struct AlignedBuffer([u8; CHUNK_SIZE]);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut ring = IoUringRuntime::new(4096)?;

    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .custom_flags(nix::libc::O_DIRECT)
        .open("Cargo.toml")?;

    let mut read_buf = Box::new(AlignedBuffer([0; CHUNK_SIZE]));
    let res = ring.read_async(file.as_raw_fd(), &mut read_buf.0).await;

    println!("read res: {res}");
    println!("{}", std::str::from_utf8(&read_buf.0[..res as usize])?);

    Ok(())
}
