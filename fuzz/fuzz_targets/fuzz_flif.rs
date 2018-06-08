#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate flif;

use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let _ = flif::Flif::decode(Cursor::new(data)).map(|img| img.get_raw_pixels());
});
