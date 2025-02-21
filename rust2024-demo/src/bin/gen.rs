#![feature(gen_blocks)]

fn to_gen<T>(slice: &[T]) -> impl Iterator<Item = &T> {
    gen move {
        for item in slice {
            yield item;
        }
    }
}

fn main() {
    let slice = [1, 2, 3, 4, 5];
    let iter = to_gen(&slice);
    for item in iter {
        println!("{}", item);
    }
}
