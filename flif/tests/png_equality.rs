extern crate png;
extern crate flif;

use std::fs::File;
use flif::Flif;
use std::io::BufReader;

#[test]
fn sea_snail() {
    let decoder = png::Decoder::new(File::open("../resources/sea_snail.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buf).unwrap();

    let file = BufReader::new(File::open("../resources/sea_snail.flif").unwrap());
    let image = Flif::decode(file).unwrap();
    let data = image.get_raw_pixels();

    assert_eq!(buf, data);
}

#[test]
fn sea_snail_cutout() {
    let decoder = png::Decoder::new(File::open("../resources/sea_snail_cutout.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buf).unwrap();

    let file = BufReader::new(File::open("../resources/sea_snail_cutout.flif").unwrap());
    let image = Flif::decode(file).unwrap();
    let data = image.get_raw_pixels();

    assert_eq!(buf, data);
}

#[test]
fn flif_logo() {
    let decoder = png::Decoder::new(File::open("../resources/flif_logo.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buf).unwrap();

    let file = BufReader::new(File::open("../resources/flif_logo.flif").unwrap());
    let image = Flif::decode(file).unwrap();
    let data = image.get_raw_pixels();

    assert_eq!(buf, data);
}
