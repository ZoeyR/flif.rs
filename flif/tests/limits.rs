extern crate flif;

use std::fs::File;
use std::io::BufReader;

use flif::Error;
use flif::Flif;
use flif::Limits;

#[test]
fn maniac_size_limit() {
    let file = BufReader::new(File::open("../resources/sea_snail.flif").unwrap());
    let mut limits: Limits = Default::default();
    limits.maniac_nodes = 16;
    match Flif::decode_with_limits(file, limits) {
        Err(Error::LimitViolation(ref message)) if message.contains("maniac") => {}
        Err(err) => panic!(
            "Expected an Error::LimitViolation indicating the maniac tree was too large, got {:?}",
            err
        ),
        _ => panic!("Expected an Error::LimitViolation indicating the maniac tree was too large, got a valid image instead")
    }
}
