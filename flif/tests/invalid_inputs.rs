extern crate flif;

use flif::Decoder;
use flif::Error;
use flif::Flif;

/// Tests an issue found in [#15](https://github.com/dgriffen/flif.rs/issues/15)
#[test]
fn invalid_bytes_per_channel() {
    let bytes = [0x46, 0x4c, 0x49, 0x46, 0x44, 0x27, 0x46, 0x46];
    let decoder = Decoder::new(bytes.as_ref());
    match decoder {
        Err(Error::InvalidHeader {
            desc: "bytes per channel was not a valid value",
        }) => {}
        _ => panic!("expected an Error::InvalidHeader indicating bytes per channel was not valid"),
    }
}

/// Tests an issue found in [#30](https://github.com/dgriffen/flif.rs/issues/30)
#[test]
fn ycocg_stack_overflow() {
    let bytes = b"FLIF41\x02\x01\x00pr@\x015\xc6\xe3d\xbfct\x00i\x005FLI)F\xca\xcdi\x00r\x00\xfft\x11-FLIF12i\x00r\x00\xfft\x11\x00\xfft\x11-FLIF12i\x00r\x00\xfft\x11-le\x00FLI 11\xe3d\xbfct\x00i\xf9\xf9\x07\xff5\xff\x00\x00";
    let limits = flif::Limits {
        metadata_chunk: 32,
        metadata_count: 8,
        pixels: 1 << 16,
        maniac_nodes: 512,
    };
    let _ = Flif::decode_with_limits(bytes.as_ref(), limits).map(|img| img.get_raw_pixels());
}

/// Tests an issue found in [#34](https://github.com/dgriffen/flif.rs/issues/34)
#[test]
fn memory_growth() {
    let bytes = b"FLIF11F\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00FLIF\x00\x00L\xc5XifI\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00FLIF\x00\x00\x00\x00\x00\x00\x00";
    let limits = flif::Limits {
        metadata_chunk: 32,
        metadata_count: 8,
        pixels: 1 << 16,
        maniac_nodes: 512,
    };
    match Flif::decode_with_limits(bytes.as_ref(), limits) {
        Err(Error::InvalidOperation(ref message)) if message.contains("maniac") => {}
        Err(err) => panic!(
            "Expected an Error::InvalidOperation indicating the maniac tree was too large, got {:?}",
            err
        ),
        _ => panic!("Expected an Error::InvalidOperation indicating the maniac tree was too large, got a valid image instead")
    }
}
