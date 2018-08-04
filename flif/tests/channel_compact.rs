extern crate flif;

use std::fs::File;
use std::io::BufReader;

use flif::components::Transformation;
use flif::Decoder;

// #[test]
// fn invalid_tree() {
//     let file = BufReader::new(File::open("../resources/invalid_tree.flif").unwrap());
//     let image = Decoder::(file).unwrap();
//     let info = image.info();

//     let expected = vec![Transformation::ChannelCompact];
//     assert_eq!(expected, info.second_header.transformations);
// }

// #[test]
// fn invalid_transform() {
//     let file = BufReader::new(File::open("../resources/invalid_tid.flif").unwrap());
//     let image = Decoder::new(file).unwrap();
//     let info = image.info();

//     let expected = vec![Transformation::ChannelCompact];
//     assert_eq!(expected, info.second_header.transformations);
// }
