extern crate flif;
extern crate png;

use std::fs::File;
use std::io::{BufReader, BufWriter};

use flif::colors::ColorSpace;
use flif::Flif;
use png::HasParameters;

fn main() {
    // decode_and_write(
    //     "resources/flif_logo.flif",
    //     "flif/examples/flif_logo_out.png",
    // );
    decode_and_write(
        "resources/sea_snail_cutout.flif",
        "flif/examples/sea_snail.png",
    );
}

fn decode_and_write(input: &str, output: &str) {
    let file = std::fs::File::open(input).unwrap();
    let reader = BufReader::new(file);

    let image = Flif::decode(reader).unwrap();

    let info = image.info();
    println!("Large Flif Info:");
    println!("├───{:?}", info.header);
    println!("├───{:?}", info.metadata);
    println!("└───{:?}", info.second_header);

    let file = File::create(output).unwrap();
    let w = &mut BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, info.header.width as u32, info.header.height as u32);

    let color_type = match info.header.channels {
        ColorSpace::RGBA => png::ColorType::RGBA,
        ColorSpace::RGB => png::ColorType::RGB,
        ColorSpace::Monochrome => png::ColorType::Grayscale,
    };

    encoder.set(color_type).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    // Get the raw pixel array of the FLIF image
    let data = image.raw();
    writer.write_image_data(data).unwrap(); // Save
}
