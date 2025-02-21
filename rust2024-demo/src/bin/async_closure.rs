use futures::executor::block_on;

fn main() {
    let work = async || {
        println!("42");
    };

    block_on(work());
}
