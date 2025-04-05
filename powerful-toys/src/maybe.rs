#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Maybe<T> {
    Just(T),
    Nothing,
}

pub use Maybe::*;

impl<T> Maybe<T> {
    pub fn map<R, F>(self, f: F) -> Maybe<R>
    where
        F: FnOnce(T) -> R,
    {
        match self {
            Just(x) => Just(f(x)),
            Nothing => Nothing,
        }
    }

    pub async fn map_async<U, F, Fut>(self, f: F) -> Maybe<U>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = U>,
    {
        match self {
            Just(x) => Just(f(x).await),
            Nothing => Nothing,
        }
    }

    pub fn and_then<R, F>(self, f: F) -> Maybe<R>
    where
        F: FnOnce(T) -> Maybe<R>,
    {
        match self {
            Just(x) => f(x),
            Nothing => Nothing,
        }
    }

    pub async fn and_then_async<R, F, Fut>(self, f: F) -> Maybe<R>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = Maybe<R>>,
    {
        match self {
            Just(x) => f(x).await,
            Nothing => Nothing,
        }
    }

    pub fn or_else<F>(self, f: F) -> Maybe<T>
    where
        F: FnOnce() -> Maybe<T>,
    {
        match self {
            Just(_) => self,
            Nothing => f(),
        }
    }
    pub async fn or_else_async<F, Fut>(self, f: F) -> Maybe<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Maybe<T>>,
    {
        match self {
            Just(_) => self,
            Nothing => f().await,
        }
    }

    pub fn as_ref(&self) -> Maybe<&T> {
        match self {
            Just(x) => Just(x),
            Nothing => Nothing,
        }
    }

    pub fn as_mut(&mut self) -> Maybe<&mut T> {
        match self {
            Just(x) => Just(x),
            Nothing => Nothing,
        }
    }

    pub fn get(self) -> T {
        match self {
            Just(x) => x,
            Nothing => panic!("called `Maybe::get()` on a `Nothing` value"),
        }
    }
}

impl<T> Maybe<Maybe<T>> {
    pub fn join(self) -> Maybe<T> {
        match self {
            Just(x) => x,
            Nothing => Nothing,
        }
    }
}
