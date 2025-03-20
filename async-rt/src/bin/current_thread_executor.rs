use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    time::{Duration, Instant},
};

type TaskQueue = Arc<Mutex<VecDeque<Pin<Box<dyn Future<Output = ()> + Send>>>>>;

pub struct Executor {
    queue: TaskQueue,
}

static VTABLE: RawWakerVTable = {
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &VTABLE)
    }

    unsafe fn wake(_data: *const ()) {
        println!("call wake")
    }

    unsafe fn wake_by_ref(_: *const ()) {}

    unsafe fn drop(_: *const ()) {}

    RawWakerVTable::new(clone, wake, wake_by_ref, drop)
};

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn create_waker(&self) -> Waker {
        let data = Arc::into_raw(self.queue.clone()) as *const ();
        unsafe { Waker::from_raw(RawWaker::new(data, &VTABLE)) }
    }

    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        self.queue.lock().unwrap().push_back(Box::pin(future));
    }

    pub fn run(&self) {
        while let Some(mut task) = { self.queue.lock().unwrap().pop_front() } {
            let waker = self.create_waker();
            let mut context = Context::from_waker(&waker);

            match task.as_mut().poll(&mut context) {
                Poll::Ready(_) => println!("task finished"),
                Poll::Pending => {
                    self.queue.lock().unwrap().push_back(task);
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        }
    }
}

async fn delay_task(id: usize) {
    println!("task {} started", id);

    #[pin_project::pin_project]
    struct DelayFuture {
        id: usize,
        start: Instant,
        duration: Duration,
    }

    impl DelayFuture {
        fn new(id: usize, duration: Duration) -> Self {
            let start = Instant::now();

            Self {
                id,
                start,
                duration,
            }
        }
    }

    impl Future for DelayFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            println!("[task {}] Polled delay", self.id);

            if Instant::now() - self.start >= self.duration {
                println!("[task {}] Delay finished", self.id);
                Poll::Ready(())
            } else {
                println!("[task {}] Delay not finished", self.id);
                Poll::Pending
            }
        }
    }
    DelayFuture::new(id, Duration::from_secs(1)).await;

    println!("task {} finished", id);
}

fn main() {
    let executor = Executor::new();

    println!("run same future");
    executor.spawn(async {
        delay_task(1).await;
        delay_task(2).await;
    });
    executor.run();

    println!("run different future");
    executor.spawn(delay_task(1));
    executor.spawn(delay_task(2));
    executor.run();
}
