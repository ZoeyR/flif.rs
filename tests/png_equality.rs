extern crate png;
extern crate flif;

use std::fs::File;
use flif::Decoder;

#[test]
fn sea_snail() {
    let decoder = png::Decoder::new(File::open("resources/sea_snail.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buf).unwrap();

    let file = std::fs::File::open("resources/sea_snail.flif").unwrap();

    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    let data = flif.get_raw_pixels();

    assert_eq!(buf, data);
}

#[test]
fn flif_logo() {
    let decoder = png::Decoder::new(File::open("resources/flif_logo.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buf).unwrap();

    let file = std::fs::File::open("resources/flif_logo.flif").unwrap();

    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    let data = flif.get_raw_pixels();

    assert_eq!(buf, data);
}