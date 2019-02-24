extern crate flif;
extern crate png;

use flif::Flif;

fn decode_png(png_data: &[u8]) -> Box<[u8]> {
    let decoder = png::Decoder::new(png_data);
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();
    buf.into_boxed_slice()
}

macro_rules! test_equality {
    ($name:ident) => {
        #[test]
        fn $name() {
            let png_data =
                include_bytes!(concat!("../../resources/", stringify!($name), ".png")).as_ref();
            let flif_data =
                include_bytes!(concat!("../../resources/", stringify!($name), ".flif")).as_ref();
            let image = Flif::decode(flif_data).unwrap();
            assert!(decode_png(png_data) == image.into_raw());
        }
    };
}

test_equality!(sea_snail);
test_equality!(sea_snail_cutout);
test_equality!(rust_logo);
test_equality!(rgba_edge);
test_equality!(road);
test_equality!(road2);
