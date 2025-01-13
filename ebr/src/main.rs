use std::{
    fmt::Debug,
    mem::ManuallyDrop,
    sync::{Arc, atomic::Ordering as AtomicOrdering},
    thread,
};

use crossbeam::epoch::{self, Atomic, Owned};

pub struct TreiberStack<T> {
    head: Atomic<Node<T>>,
}

struct Node<T> {
    elem: ManuallyDrop<T>,
    next: Atomic<Node<T>>,
}

impl<T> Debug for Node<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("elem", &self.elem)
            .field("next", &self.next)
            .finish()
    }
}

impl<T> Debug for TreiberStack<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreiberStack")
            .field("head", &self.head)
            .finish()
    }
}

impl<T> TreiberStack<T> {
    pub fn new() -> Self {
        Self {
            head: Atomic::null(),
        }
    }

    pub fn push(&self, elem: T) {
        let mut node = Owned::new(Node {
            elem: ManuallyDrop::new(elem),
            next: Atomic::null(),
        });

        let guard = epoch::pin();

        loop {
            let head = self.head.load(AtomicOrdering::Relaxed, &guard);
            node.next.store(head, AtomicOrdering::Relaxed);

            match self.head.compare_exchange(
                head,
                node,
                AtomicOrdering::Release,
                AtomicOrdering::Relaxed,
                &guard,
            ) {
                Ok(_) => break,
                Err(e) => node = e.new,
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        let guard = epoch::pin();

        loop {
            let head = self.head.load(AtomicOrdering::Acquire, &guard);

            match unsafe { head.as_ref() } {
                Some(h) => {
                    let next = h.next.load(AtomicOrdering::Relaxed, &guard);

                    if self
                        .head
                        .compare_exchange(
                            head,
                            next,
                            AtomicOrdering::Relaxed,
                            AtomicOrdering::Relaxed,
                            &guard,
                        )
                        .is_ok()
                    {
                        unsafe {
                            guard.defer_destroy(head);

                            let elem = ManuallyDrop::into_inner(std::ptr::read(&(*h).elem));
                            return Some(elem);
                        }
                    }
                }
                None => return None,
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head
            .load(AtomicOrdering::Acquire, &epoch::pin())
            .is_null()
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::*;

    #[test]
    fn test_push_pop() {
        let stack = TreiberStack::new();
        assert!(stack.is_empty());

        stack.push(1);
        assert!(!stack.is_empty());

        assert_eq!(stack.pop(), Some(1));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_multiple_push_pop() {
        let stack = TreiberStack::new();
        assert!(stack.is_empty());

        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_concurrent_push_pop() {
        let stack = Arc::new(TreiberStack::new());
        let mut handles = vec![];

        for i in 0..4 {
            let stack = Arc::clone(&stack);
            handles.push(thread::spawn(move || {
                for j in 0..25 {
                    stack.push(i * 25 + j);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut results = vec![];
        for _ in 0..100 {
            if let Some(value) = stack.pop() {
                results.push(value);
            }
        }

        results.sort();
        assert_eq!(results, (0..100).collect::<Vec<_>>());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let stack = TreiberStack::new();
        assert!(stack.is_empty());

        stack.push(1);
        assert!(!stack.is_empty());

        stack.pop();
        assert!(stack.is_empty());
    }
}

fn main() {
    let stack = Arc::new(TreiberStack::new());
    let mut handles = vec![];

    for i in 0..4 {
        let stack = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            for j in 0..25 {
                stack.push(i * 25 + j);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut results = vec![];
    for _ in 0..100 {
        if let Some(value) = stack.pop() {
            results.push(value);
        }
    }

    results.sort();
    assert_eq!(results, (0..100).collect::<Vec<_>>());
    assert!(stack.is_empty());
}
