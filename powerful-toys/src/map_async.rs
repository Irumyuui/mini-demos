use std::{
    pin::Pin,
    task::{Context, Poll, ready},
};

use pin_project::pin_project;

pub trait FutureExt: Future {
    fn map_async<F, U>(self, f: F) -> MapFuture<Self, F>
    where
        Self: Sized + Send,
        F: FnOnce(Self::Output) -> U + Send,
    {
        map_async(self, f)
    }
}

impl<Fut: Future> FutureExt for Fut {}

fn map_async<Fut, F, T, U>(fut: Fut, f: F) -> MapFuture<Fut, F>
where
    Fut: Future<Output = T> + Send,
    F: FnOnce(T) -> U + Send,
{
    MapFuture::new(fut, f)
}

#[pin_project]
pub struct MapFuture<Fut, F> {
    #[pin]
    future: Fut,
    f: Option<F>,
}

impl<Fut, F, T, U> MapFuture<Fut, F>
where
    Fut: Future<Output = T> + Send,
    F: FnOnce(T) -> U + Send,
{
    fn new(future: Fut, f: F) -> Self {
        MapFuture { future, f: Some(f) }
    }
}

impl<Fut, F, T, U> Future for MapFuture<Fut, F>
where
    Fut: Future<Output = T> + Send,
    F: FnOnce(T) -> U + Send,
{
    type Output = U;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let output = ready!(this.future.poll(cx));
        Poll::Ready(this.f.take().expect("may be call twice?")(output))
    }
}

#[cfg(test)]
mod tests {
    use crate::map_async::FutureExt;

    #[tokio::test]
    async fn it_work() {
        let foo = async || 42;
        let result = foo().map_async(|x| x + 1).await;
        assert_eq!(result, 43);
    }
}
