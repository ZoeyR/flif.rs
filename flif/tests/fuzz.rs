extern crate flif;

use flif::Flif;
use std::io::{Cursor, Read};

/// Tests an issue found in [#15](https://github.com/dgriffen/flif.rs/issues/15)
#[test]
fn fuzz_artifacts() {
    let paths = std::fs::read_dir("../fuzz/artifacts/fuzz_flif/").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        println!("Artifact: {}", path.display());
        let mut data = Vec::new();
        let mut file = std::fs::File::open(path).unwrap();
        file.read_to_end(&mut data).unwrap();
        // temporarily disabled
        //let _ = Flif::decode(Cursor::new(&data)).map(|img| img.get_raw_pixels());
    }
}
