extern crate flif;

use std::fs::File;
use std::io::BufReader;

use flif::components::Transformation;
use flif::Decoder;

#[test]
fn invalid_tree() {
    let file = BufReader::new(File::open("../resources/invalid_tree.flif").unwrap());
    let image = Decoder::new(file).unwrap();
    let info = image.info();

    let expected = vec!["Original (Pseudo Transformation)", "Channel Compact"];
    assert_eq!(
        expected,
        info.second_header
            .transformations
            .set
            .iter()
            .map(|t| format!("{}", t))
            .collect::<Vec<_>>()
    );
}

#[test]
fn invalid_transform() {
    let file = BufReader::new(File::open("../resources/invalid_tid.flif").unwrap());
    let image = Decoder::new(file).unwrap();
    let info = image.info();

    let expected = vec!["Original (Pseudo Transformation)", "Channel Compact"];
    assert_eq!(
        expected,
        info.second_header
            .transformations
            .set
            .iter()
            .map(|t| format!("{}", t))
            .collect::<Vec<_>>()
    );
}
