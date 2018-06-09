#![feature(test)]
extern crate flif;
extern crate test;

use flif::Flif;
use test::Bencher;

use std::io::{Cursor, Read};

#[bench]
fn bench_cutout_full_decode(b: &mut Bencher) {
    let mut data = Vec::new();
    let mut file = std::fs::File::open("../resources/sea_snail_cutout.flif").unwrap();
    file.read_to_end(&mut data).unwrap();
    b.iter(|| {
        let raw = Flif::decode(Cursor::new(&data)).unwrap().get_raw_pixels();
        test::black_box(raw);
    });
}
