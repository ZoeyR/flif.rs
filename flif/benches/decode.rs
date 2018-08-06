#![feature(test)]
extern crate flif;
extern crate test;

use flif::Flif;
use test::Bencher;

#[bench]
fn bench_cutout_full_decode(b: &mut Bencher) {
    let data = include_bytes!("../../resources/sea_snail_cutout.flif");
    b.iter(|| {
        let img = Flif::decode(data.as_ref()).unwrap();
        test::black_box(img.raw());
    });
}

#[bench]
fn bench_grey_decode(b: &mut Bencher) {
    let data = include_bytes!("../../resources/road.flif");
    b.iter(|| {
        let img = Flif::decode(data.as_ref()).unwrap();
        test::black_box(img.raw());
    });
}
