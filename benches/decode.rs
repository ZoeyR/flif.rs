#![feature(test)]
extern crate png;
extern crate flif;
extern crate test;

use test::Bencher;
use flif::Decoder;

fn cutout_full_decode() {
    let file = std::fs::File::open("resources/sea_snail_cutout.flif").unwrap();
    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    flif.get_raw_pixels();
}

#[bench]
fn bench_cutout_full_decode(b: &mut Bencher) {
    b.iter(|| cutout_full_decode());
}
