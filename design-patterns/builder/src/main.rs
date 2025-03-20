use std::io::Read;

fn main() {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("Cargo.toml")
        .expect("Unable to open file");

    let mut content = String::new();
    file.read_to_string(&mut content).expect("io error");

    println!("{}", content);
}
