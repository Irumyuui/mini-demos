#![feature(let_chains)]

fn main() {
    let opt = Some(42);
    if let Some(x) = opt
        && x == 42
    {
        println!("{}", x);
    }
}
