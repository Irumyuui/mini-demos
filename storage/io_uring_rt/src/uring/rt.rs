use std::time::Duration;
use std::{cell::RefCell, future::poll_fn, mem::ManuallyDrop, rc::Rc};

use tokio::io::unix::AsyncFd;
use tokio::task::LocalSet;

use rustix_uring::Builder as IoUringBuilder;
use tokio::runtime::Builder as TokioRtBuiler;
use tokio::runtime::Runtime as TokioRuntime;
use tokio::time::Instant;
use tracing::field::debug;

use super::driver::CONTEXT;
use super::driver::Handle;
use crate::uring::driver::Driver;

pub struct Runtime {
    rt: TokioRuntime,
    local: LocalSet,
    driver: Rc<RefCell<Driver>>,
}

impl Runtime {
    pub fn new(
        // mut rt_builder: tokio::runtime::Builder,
        uring_buidler: &IoUringBuilder,
        entries: u32,
    ) -> std::io::Result<Self> {
        let rt = TokioRtBuiler::new_current_thread()
            .on_thread_park(|| {
                CONTEXT.with(|c| {
                    let _ = c
                        .handle()
                        .expect("not found io uring context, is it init?")
                        .borrow_mut()
                        .uring
                        .submit();
                });
            })
            .enable_all()
            .build()?;

        let local = LocalSet::new();
        let driver = Rc::new(RefCell::new(Driver::new(uring_buidler, entries)?));

        wake_uring_task(
            &rt,
            &local,
            Handle {
                inner: driver.clone(),
            },
        );

        Ok(Self { rt, local, driver })
    }

    pub fn block_on<Fut: Future>(&self, fut: Fut) -> Fut::Output {
        tracing::debug!("into block task");

        struct ContextGuard;

        impl Drop for ContextGuard {
            fn drop(&mut self) {
                CONTEXT.with(|c| c.unset());
            }
        }

        CONTEXT.with(|c| c.set(self.driver.clone()));
        let _g = ContextGuard;

        tokio::pin!(fut);

        let fut = poll_fn(|cx| fut.as_mut().poll(cx));
        let fut = self.local.run_until(fut);

        self.rt.block_on(fut)
    }
}

fn wake_uring_task(rt: &TokioRuntime, local: &LocalSet, driver: Handle) {
    let _guard = rt.enter();
    let handle = AsyncFd::new(driver).unwrap();

    local.spawn_local(async move {
        loop {
            let mut guard = handle.readable().await.unwrap();
            guard.get_inner().inner.borrow_mut().tick();
            guard.clear_ready();
        }
    });
}

pub fn default_rt() -> std::io::Result<Runtime> {
    Runtime::new(&rustix_uring::IoUring::builder(), 256)
}

#[cfg(test)]
mod tests {
    use std::{io::Write, rc::Rc, time::Duration};

    use tempfile::tempfile;

    use crate::uring::fs::File;

    use super::{Runtime, default_rt};

    #[test]
    fn test_block_on() {
        let rt = default_rt().unwrap();
        rt.block_on(async {
            tokio::time::sleep(Duration::from_micros(100)).await;
        });
    }

    #[test]
    fn test_block_twice() {
        let rt = default_rt().unwrap();
        rt.block_on(async {
            tokio::time::sleep(Duration::from_micros(100)).await;
        });
        rt.block_on(async {
            tokio::time::sleep(Duration::from_micros(100)).await;
        });
    }

    #[test]
    fn test_write_and_read() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        let rt = default_rt().unwrap();

        rt.block_on(async move {
            let mut file = tempfile().unwrap();
            file.write_all(b"hello world").unwrap();

            let file = File::from_std_fd(file);
            let file = Rc::new(file);

            let f1 = file.clone();
            let f2 = file.clone();

            let h1 = tokio::task::spawn_local(async move {
                let mut buf = vec![0; 5];
                let (res, buf) = f1.read_at(buf, 0).await;
                (res, buf)
            });
            let h2 = tokio::task::spawn_local(async move {
                let mut buf = vec![0; 6];
                let (res, buf) = f2.read_at(buf, 5).await;
                (res, buf)
            });

            let (res1, buf1) = h1.await.unwrap();
            let (res2, buf2) = h2.await.unwrap();

            eprintln!("{:?}", String::from_utf8(buf1));
            eprintln!("{:?}", String::from_utf8(buf2));
        });
    }
}
