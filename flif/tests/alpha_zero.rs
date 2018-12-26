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

#[test]
fn alpha_zero_test() {
    let png_data = include_bytes!("../../resources/rust_logo.png").as_ref();
    let flif_data = include_bytes!(
        "../../resources/rust_logo_discard_invisible.flif"
    ).as_ref();
    let png_frame = decode_png(png_data);
    let flif_frame = Flif::decode(flif_data).unwrap().into_raw();
    for (p, f) in png_frame.chunks(4).zip(flif_frame.chunks(4)) {
        assert_eq!(p[3], f[3]);
        if p[3] == 0 { continue; }
        assert_eq!(&p[..3], &f[..3]);
    }
}