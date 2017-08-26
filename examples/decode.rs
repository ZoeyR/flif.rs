extern crate flif;

use flif::Decoder;

fn main() {
    let file = std::fs::File::open("examples/flif.flif").unwrap();

    let mut decoder = Decoder::new(file);
    let header = decoder.read_main_header().unwrap();
    println!("{:?}", header);
}