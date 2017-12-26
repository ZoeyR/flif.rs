#![feature(test)]
extern crate png;
extern crate flif;
extern crate test;

use test::Bencher;
use flif::Decoder;

fn logo_full_decode() {
    let file = std::fs::File::open("resources/flif_logo.flif").unwrap();
    let mut decoder = Decoder::new(file);
    let flif = decoder.decode().unwrap();
    flif.get_raw_pixels();
}

#[bench]
fn bench_logo_full_decode(b: &mut Bencher) {
    b.iter(|| logo_full_decode());
}
