extern crate flif;
extern crate png;

use flif::components::header::Channels;
use flif::Decoder;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use png::HasParameters;

fn main() {
    decode_and_write("resources/flif_logo.flif", "examples/flif_logo_out.png");
    decode_and_write("resources/sea_snail.flif", "examples/sea_snail_out.png");
}

fn decode_and_write(input: &str, output: &str) {
    let file = std::fs::File::open("resources/sea_snail.flif").unwrap();

    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    println!("Large Flif Info:");
    println!("├───{:?}", flif.info.header);
    println!("├───{:?}", flif.info.metadata);
    println!("└───{:?}", flif.info.second_header);

    let path = Path::new("resources/out.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, flif.info.header.width as u32, flif.info.header.height as u32);

    let color_type = match flif.info.header.channels {
        Channels::RGBA => png::ColorType::RGBA,
        Channels::RGB => png::ColorType::RGB,
        _ => panic!("unsupported color type"),
    };

    encoder.set(color_type).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    let data = flif.get_raw_pixels(); // Get the raw pixel array of the FLIF image
    writer.write_image_data(&data).unwrap(); // Save
}