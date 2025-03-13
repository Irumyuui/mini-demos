use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

// pub struct TimeWrapper<Fut: Future> {
//     start: Option<Instant>,
//     future: Fut,
// }

// impl<Fut: Future> TimeWrapper<Fut> {
//     pub fn new(future: Fut) -> Self {
//         Self {
//             start: None,
//             future,
//         }
//     }
// }

// impl<Fut: Future> Future for TimeWrapper<Fut> {
// type Output = (Fut::Output, Duration);

//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let start = self.start.get_or_insert_with(Instant::now);
//         let inner_poll = self.future.poll(cx); // Should Pin
//         let elapsed = self.elapsed();

//         match inner_poll {
//             Poll::Pending => Poll::Pending,
//             Poll::Ready(output) => Poll::Ready((output, elapsed)),
//         }
//     }
// }

#[pin_project::pin_project]
pub struct TimeWrapper<Fut: Future> {
    start: Option<Instant>,

    #[pin]
    future: Fut,
}

impl<Fut: Future> TimeWrapper<Fut> {
    pub fn new(future: Fut) -> Self {
        Self {
            start: None,
            future,
        }
    }
}

impl<Fut: Future> Future for TimeWrapper<Fut> {
    type Output = (Fut::Output, Duration);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let start = this.start.get_or_insert_with(Instant::now);
        let inner_poll = this.future.poll(cx);
        let elapsed = Instant::now().duration_since(*start);

        match inner_poll {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready((output, elapsed)),
        }
    }
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let rt = Arc::new(rt);

    futures::executor::block_on(async {
        let future = async {
            rt.block_on(async {
                tokio::time::sleep(Duration::from_secs(1)).await;
            });
            42
        };

        let timed_future = TimeWrapper::new(future);
        let (output, elapsed) = timed_future.await;
        println!("Output: {}, Elapsed: {:?}", output, elapsed);
    });
}
