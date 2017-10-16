extern crate flif;

use flif::Decoder;

fn main() {
    let file = std::fs::File::open("examples/flif_small.flif").unwrap();

    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    println!("Compact Flif Info:");
    println!("├───{:?}", flif.header);
    println!("├───{:?}", flif.metadata);
    println!("└───{:?}", flif.second_header);

    let file = std::fs::File::open("examples/flif_large.flif").unwrap();

    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    println!("Large Flif Info:");
    println!("├───{:?}", flif.header);
    println!("├───{:?}", flif.metadata);
    println!("└───{:?}", flif.second_header);
}