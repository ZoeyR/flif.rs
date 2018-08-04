extern crate flif;
extern crate png;

use std::fs::File;
use std::io::BufReader;

use flif::Flif;

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

#[test]
fn road() {
    let decoder = png::Decoder::new(File::open("../resources/road.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buf).unwrap();

    let file = BufReader::new(File::open("../resources/road.flif").unwrap());
    let image = Flif::decode(file).unwrap();
    let data = image.get_raw_pixels();

    assert_eq!(buf, data);
}

#[test]
fn road2() {
    let decoder = png::Decoder::new(File::open("../resources/road2.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buf).unwrap();

    let file = BufReader::new(File::open("../resources/road2.flif").unwrap());
    let image = Flif::decode(file).unwrap();
    let data = image.get_raw_pixels();

    assert_eq!(buf[..4], data[..4]);
}
