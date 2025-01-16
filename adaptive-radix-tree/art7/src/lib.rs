mod art_impl;
mod node;

pub use art_impl::Art;

#[cfg(test)]
mod tests {
    use crate::Art;

    #[test]
    fn insert_and_get() {
        let mut art = Art::new();

        art.insert(b"hello".to_vec(), b"world".to_vec());
        assert_eq!(art.get(b"hello"), Some(b"world".as_slice()));
    }
}
