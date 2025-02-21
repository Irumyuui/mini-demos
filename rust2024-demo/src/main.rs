fn indices<T>(slice: &[T]) -> impl Iterator<Item = usize> + use<T> {
    0..slice.len()
}

fn main() {
    let mut data = vec![1, 2, 3];
    let iter = indices(&data);
    data.push(4);
    println!("{:?}", iter.collect::<Vec<_>>());
    println!("{:?}", data);
}
