#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate flif;

use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let limits = flif::Limits {
        metadata_chunk: 32,
        metadata_count: 8,
        pixels: 1<<16,
        maniac_nodes: 512,
    };
    let _ = flif::Flif::decode_with_limits(Cursor::new(data), limits)
        .map(|img| img.into_raw());
});
