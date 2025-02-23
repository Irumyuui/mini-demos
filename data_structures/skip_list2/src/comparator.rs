use std::{cmp, marker::PhantomData};

pub trait Comparator: Send + Sync {
    type Item;

    fn compare(&self, a: &Self::Item, b: &Self::Item) -> cmp::Ordering;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultComparator<T> {
    _marker: PhantomData<T>,
}

impl<T> Comparator for DefaultComparator<T>
where
    T: Send + Sync + Ord,
{
    type Item = T;

    fn compare(&self, a: &Self::Item, b: &Self::Item) -> cmp::Ordering {
        a.cmp(b)
    }
}
