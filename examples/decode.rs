extern crate flif;

use flif::Decoder;

fn main() {
    let file = std::fs::File::open("C:/Users/micro/Documents/flif.rs/examples/sea_snail.flif").unwrap();

    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    println!("Large Flif Info:");
    println!("├───{:?}", flif.header);
    println!("├───{:?}", flif.metadata);
    println!("└───{:?}", flif.second_header);
}