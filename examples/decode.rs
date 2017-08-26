extern crate flif;

use flif::Decoder;

fn main() {
    let file = std::fs::File::open("examples/flif.flif").unwrap();

    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    println!("{:?}", flif.header);
    println!("{:?}", flif.metadata)
}