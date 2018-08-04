extern crate flif;

use std::fs::File;
use std::io::BufReader;

use flif::Decoder;

#[test]
fn invalid_tree() {
    let file = BufReader::new(File::open("../resources/invalid_tree.flif").unwrap());
    let image = Decoder::new(file).unwrap();
    let info = image.info();

    let expected = "Channel Compact";
    assert_eq!(
        expected,
        format!("{}", info.second_header.transformations.last)
    );
}

#[test]
fn invalid_transform() {
    let file = BufReader::new(File::open("../resources/invalid_tid.flif").unwrap());
    let image = Decoder::new(file).unwrap();
    let info = image.info();

    let expected = "Channel Compact";
    assert_eq!(
        expected,
        format!("{}", info.second_header.transformations.last)
    );
}
