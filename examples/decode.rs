extern crate flif;
extern crate png;

use flif::Decoder;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use png::HasParameters;

fn main() {
    let file = std::fs::File::open("C:/Users/micro/Documents/flif.rs/examples/tiny.flif").unwrap();

    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    println!("Large Flif Info:");
    println!("├───{:?}", flif.info.header);
    println!("├───{:?}", flif.info.metadata);
    println!("└───{:?}", flif.info.second_header);

    // To use encoder.set()

    let path = Path::new(r"C:/Users/micro/Documents/flif.rs/examples/out.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, flif.info.header.width as u32, flif.info.header.height as u32); // Width is 2 pixels and height is 1.
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    let data = flif.get_raw_pixels(); // An array containing a RGBA sequence. First pixel is red and second pixel is black.
    writer.write_image_data(&data).unwrap(); // Save
}