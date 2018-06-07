extern crate flif;

use flif::Decoder;
use flif::Error;

/// Tests an issue found in [#15](https://github.com/dgriffen/flif.rs/issues/15)
#[test]
fn invalid_bytes_per_channel() {
    let bytes = [0x46,0x4c,0x49,0x46,0x44,0x27,0x46,0x46];
    let decoder = Decoder::new(bytes.as_ref());
    match decoder {
        Err(Error::InvalidHeader{desc: "bytes per channel was not a valid value"}) => {},
        _ => panic!("expected an Error::InvalidHeader indicating bytes per channel was not valid"),
    }
}
