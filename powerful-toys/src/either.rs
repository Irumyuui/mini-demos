#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub use Either::*;

impl<L, R> Either<L, R> {
    pub fn left(self) -> Option<L> {
        match self {
            Either::Left(l) => Some(l),
            Either::Right(_) => None,
        }
    }

    pub fn right(self) -> Option<R> {
        match self {
            Either::Left(_) => None,
            Either::Right(r) => Some(r),
        }
    }

    pub fn map_left<F, U>(self, f: F) -> Either<U, R>
    where
        F: FnOnce(L) -> U,
    {
        match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(r),
        }
    }

    pub fn map_right<F, U>(self, f: F) -> Either<L, U>
    where
        F: FnOnce(R) -> U,
    {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(f(r)),
        }
    }

    pub fn as_ref(&self) -> Either<&L, &R> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(r),
        }
    }

    pub fn as_mut(&mut self) -> Either<&mut L, &mut R> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(r),
        }
    }

    pub fn flip(self) -> Either<R, L> {
        match self {
            Either::Left(l) => Either::Right(l),
            Either::Right(r) => Either::Left(r),
        }
    }
}

impl<L, R> Either<Either<L, R>, R> {
    pub fn join(self) -> Either<L, R> {
        match self {
            Either::Left(Either::Left(l)) => Either::Left(l),
            Either::Left(Either::Right(r)) => Either::Right(r),
            Either::Right(r) => Either::Right(r),
        }
    }
}

impl<L, R> Either<L, Either<L, R>> {
    pub fn join(self) -> Either<L, R> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(Either::Left(l)) => Either::Left(l),
            Either::Right(Either::Right(r)) => Either::Right(r),
        }
    }
}

impl<L, R> Either<Either<L, R>, Either<L, R>> {
    pub fn join(self) -> Either<L, R> {
        match self {
            Either::Left(Either::Left(l)) => Either::Left(l),
            Either::Left(Either::Right(r)) => Either::Right(r),
            Either::Right(Either::Left(l)) => Either::Left(l),
            Either::Right(Either::Right(r)) => Either::Right(r),
        }
    }
}
