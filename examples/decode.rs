extern crate flif;

use flif::components::header::Header;

fn main() {
    let file = std::fs::File::open("examples/flif.flif").unwrap();

    let header = Header::from_reader(file).unwrap();
    println!("{:?}", header);
}