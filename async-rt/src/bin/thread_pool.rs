#![allow(unused)]

use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    pin::Pin,
    process::Output,
    sync::{Arc, OnceLock},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use crossbeam::channel::Sender;
use futures::task::{ArcWake, waker_ref};
use itertools::Itertools;
use parking_lot::{Condvar, Mutex, Once};

pub struct ThreadPollExecutor {
    sender: Sender<Arc<Task>>,
    handles: Vec<Option<std::thread::JoinHandle<()>>>,
}

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
    sender: Sender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        println!("wake_by_ref, resend task to executor");
        let this = arc_self.clone();
        arc_self.sender.send(this).unwrap();
    }
}

impl ThreadPollExecutor {
    pub fn new(num_workers: usize) -> Self {
        let (sender, receiver) = crossbeam::channel::unbounded::<Arc<Task>>();

        let handles = (0..num_workers)
            .map(|id| {
                let receiver = receiver.clone();

                std::thread::spawn(move || {
                    println!("worker {} started", id);

                    while let Ok(task) = receiver.recv() {
                        let mut future = task.future.lock();
                        let waker = waker_ref(&task);
                        let mut context = Context::from_waker(&waker);

                        match future.as_mut().poll(&mut context) {
                            Poll::Ready(()) => {
                                println!("task finished");
                            }
                            Poll::Pending => {
                                // waker will be called one more time.
                            }
                        }
                    }

                    println!("worker {} stopped", id);
                })
            })
            .map(Some)
            .collect_vec();

        Self { sender, handles }
    }

    pub fn spawn(&self, fut: impl Future<Output = ()> + Send + 'static) {
        let fut = Box::pin(fut);
        let task = Arc::new(Task {
            future: Mutex::new(fut),
            sender: self.sender.clone(),
        });
        self.sender.send(task).unwrap();
    }
}

struct TimeSchedulerExecutor {
    timers: Arc<Mutex<BinaryHeap<Reverse<Timer>>>>,
    condvar: Arc<(Mutex<()>, Condvar)>,
}

struct Timer {
    when: Instant,
    waker: Waker,
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.when, &other.when)
    }
}

impl Eq for Timer {}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.when.cmp(&other.when)
    }
}

impl TimeSchedulerExecutor {
    pub fn new() -> Self {
        let timers = Arc::new(Mutex::new(BinaryHeap::new()));
        let condvar = Arc::new((Mutex::new(()), Condvar::new()));

        let mut this = TimeSchedulerExecutor { timers, condvar };
        this.start_timer_thread();
        this
    }

    fn start_timer_thread(&self) {
        let timers = self.timers.clone();
        let condvar = self.condvar.clone();

        std::thread::spawn(move || {
            println!("timer start");

            loop {
                println!("timer tick");

                let (lock, condvar) = &*condvar;
                let mut guard = lock.lock();

                let sleep_duration = timers
                    .lock()
                    .peek()
                    .map(|Reverse(timer)| {
                        let now = Instant::now();
                        if timer.when <= now {
                            Duration::ZERO
                        } else {
                            timer.when - now
                        }
                    })
                    .unwrap_or(Duration::from_secs(1));

                let _ = condvar.wait_for(&mut guard, sleep_duration);

                let mut timers = timers.lock();
                while let Some(Reverse(timer)) = timers.peek() {
                    if timer.when > Instant::now() {
                        break;
                    }
                    let Reverse(timer) = timers.pop().unwrap();
                    timer.waker.wake();
                }
            }
        });
    }

    fn register_delay(&self, when: Instant, waker: Waker) {
        let mut timers = self.timers.lock();
        timers.push(Reverse(Timer { when, waker }));
        self.condvar.1.notify_one();
    }

    pub(crate) fn global() -> Arc<Self> {
        static THIS: OnceLock<Arc<TimeSchedulerExecutor>> = OnceLock::new();
        THIS.get_or_init(|| Arc::new(Self::new())).clone()
    }
}

pub struct DelayFuture {
    when: Instant,
    waker: Arc<Mutex<Option<Waker>>>,

    scheduler: Arc<TimeSchedulerExecutor>,
}

impl DelayFuture {
    fn new(duration: Duration) -> Self {
        Self {
            when: Instant::now() + duration,
            waker: Arc::new(Mutex::new(None)),
            scheduler: TimeSchedulerExecutor::global(),
        }
    }
}

impl Future for DelayFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            Poll::Ready(())
        } else {
            let mut waker = self.waker.lock();
            waker.replace(cx.waker().clone());
            self.scheduler.register_delay(self.when, cx.waker().clone());
            Poll::Pending
        }
    }
}

pub fn sleep(duration: Duration) -> DelayFuture {
    DelayFuture::new(duration)
}

fn main() {
    let executor = ThreadPollExecutor::new(4);
    executor.spawn(async {
        sleep(Duration::from_secs(2)).await;
        println!("delay 2s");
    });
    std::thread::sleep(Duration::from_secs(3));
}
