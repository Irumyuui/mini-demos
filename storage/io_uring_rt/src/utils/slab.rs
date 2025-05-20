use std::{mem, ops};

#[derive(Debug, Clone)]
enum Entry<T> {
    Vacant(usize),
    Occupied(T),
}

#[derive(Debug, Clone)]
pub struct Slab<T> {
    entries: Vec<Entry<T>>,
    len: usize,
    next: usize,
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Slab<T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            next: 0,
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.len = 0;
        self.next = 0;
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        match self.entries.get(key) {
            Some(Entry::Occupied(elem)) => Some(elem),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        match self.entries.get_mut(key) {
            Some(Entry::Occupied(elem)) => Some(elem),
            _ => None,
        }
    }

    pub fn insert(&mut self, elem: T) -> usize {
        let key = self.next;
        self.insert_at(key, elem);
        key
    }

    fn insert_at(&mut self, key: usize, elem: T) {
        self.len += 1;

        if key == self.entries.len() {
            self.entries.push(Entry::Occupied(elem));
            self.next = key + 1;
        } else {
            self.next = match self.entries.get(key) {
                Some(Entry::Vacant(next)) => *next,
                _ => unreachable!(),
            };
            self.entries[key] = Entry::Occupied(elem);
        }
    }

    pub fn remove(&mut self, key: usize) -> T {
        self.try_remove(key).expect("invalid key")
    }

    pub fn try_remove(&mut self, key: usize) -> Option<T> {
        match self.entries.get_mut(key) {
            Some(entry) => match mem::replace(entry, Entry::Vacant(self.next)) {
                e @ Entry::Vacant(..) => {
                    *entry = e;
                    None
                }
                Entry::Occupied(elem) => {
                    self.len -= 1;
                    self.next = key;
                    Some(elem)
                }
            },
            None => None,
        }
    }
}

impl<T> ops::Index<usize> for Slab<T> {
    type Output = T;

    fn index(&self, key: usize) -> &Self::Output {
        self.get(key).expect("invalid key")
    }
}

impl<T> ops::IndexMut<usize> for Slab<T> {
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        self.get_mut(key).expect("invalid key")
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::Slab;

    #[test]
    fn test_empty() {
        let slab = Slab::<i32>::new();
        assert!(slab.is_empty());
    }

    #[test]
    fn test_drop() {
        static COUNTER: AtomicUsize = const { AtomicUsize::new(0) };

        struct Counter;

        impl Counter {
            pub fn new() -> Self {
                COUNTER.fetch_add(1, Ordering::SeqCst);
                Self
            }
        }

        impl Drop for Counter {
            fn drop(&mut self) {
                COUNTER.fetch_sub(1, Ordering::SeqCst);
            }
        }

        let mut slab = Slab::with_capacity(10);
        assert_eq!(slab.capacity(), 10);
        for _ in 0..10 {
            slab.insert(Counter::new());
        }

        drop(slab);
        assert_eq!(COUNTER.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_capcity_grow() {
        let mut arena = Slab::with_capacity(10);

        for i in 0..10 {
            let key = arena.insert(i + 10);
            assert_eq!(arena[key], i + 10);
        }
        assert_eq!(arena.capacity(), 10);

        let key = arena.insert(20);
        assert_eq!(arena[key], 20);
        assert_eq!(arena.capacity(), 20);
    }
}
