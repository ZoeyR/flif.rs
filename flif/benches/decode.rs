#![feature(test)]
extern crate flif;
extern crate test;

use flif::Flif;
use test::Bencher;

#[bench]
fn bench_cutout_full_decode(b: &mut Bencher) {
    let data = include_bytes!("../../resources/sea_snail_cutout.flif");
    b.iter(|| {
        let raw = Flif::decode(data.as_ref()).unwrap().get_raw_pixels();
        test::black_box(raw);
    });
}

#[bench]
fn bench_grey_decode(b: &mut Bencher) {
    let data = include_bytes!("../../resources/road.flif");
    b.iter(|| {
        let raw = Flif::decode(data.as_ref()).unwrap().get_raw_pixels();
        test::black_box(raw);
    });
}

/*
#[bench]
fn bench_full(b: &mut Bencher) {
    let data = include_bytes!("../../resources/invalid_tid.flif");
    b.iter(|| {
        let raw = Flif::decode(data.as_ref()).unwrap().get_raw_pixels();
        test::black_box(raw);
    });
}
*/