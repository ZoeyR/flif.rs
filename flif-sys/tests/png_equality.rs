extern crate flif_sys as flif;
extern crate png;
extern crate libc;

fn decode_gray(data: &[u8]) -> Box<[u8]> {
    unsafe {
        let decoder = flif::flif_create_decoder();
        let res = flif::flif_decoder_decode_memory(
            decoder,
            data.as_ptr() as *const libc::c_void,
            data.len(),
        );
        assert_eq!(res, 1);
        let info = flif::flif_read_info_from_memory(
            data.as_ptr() as *const libc::c_void,
            data.len(),
        );
        assert!(!info.is_null());

        let w = flif::flif_info_get_width(info);
        let h = flif::flif_info_get_height(info);
        let chans = flif::flif_info_get_nb_channels(info);
        assert_eq!(chans, 1);
        let depth = flif::flif_info_get_depth(info);
        assert_eq!(depth, 8);
        let images = flif::flif_decoder_num_images(decoder);
        assert_eq!(images, 1);
        flif::flif_destroy_info(info);

        let mut buf = vec![0u8; (w*h) as usize];
        let image = flif::flif_decoder_get_image(decoder, 0);
        for i in 0..h {
            let ptr = buf.as_mut_ptr().offset((i*w) as isize)
                as *mut libc::c_void;
            flif::flif_image_read_row_GRAY8(image, i, ptr, w as usize);
        }
        buf.into_boxed_slice()
    }
}

fn decode_rgba(data: &[u8]) -> Box<[u8]> {
    unsafe {
        let decoder = flif::flif_create_decoder();
        let res = flif::flif_decoder_decode_memory(
            decoder,
            data.as_ptr() as *const libc::c_void,
            data.len(),
        );
        assert_eq!(res, 1);
        let info = flif::flif_read_info_from_memory(
            data.as_ptr() as *const libc::c_void,
            data.len(),
        );
        assert!(!info.is_null());

        let w = flif::flif_info_get_width(info);
        let h = flif::flif_info_get_height(info);
        let chans = flif::flif_info_get_nb_channels(info);
        assert!(chans == 3 || chans == 4);
        let depth = flif::flif_info_get_depth(info);
        assert_eq!(depth, 8);
        let images = flif::flif_decoder_num_images(decoder);
        assert_eq!(images, 1);
        flif::flif_destroy_info(info);

        let mut buf = vec![0u8; 4*(w*h) as usize];
        let image = flif::flif_decoder_get_image(decoder, 0);
        for i in 0..h {
            let ptr = buf.as_mut_ptr().offset((4*i*w) as isize)
                as *mut libc::c_void;
            flif::flif_image_read_row_RGBA8(image, i, ptr, 4*w as usize);
        }
        buf.into_boxed_slice()
    }
}

fn compare_rgb_rgba(rgb: &[u8], rgba: &[u8]) {
    assert_eq!(rgb.len() / 3, rgba.len() / 4);
    assert_eq!(rgb.len() % 3, 0);
    assert_eq!(rgba.len() % 4, 0);
    for (p1, p2) in rgb.chunks(3).zip(rgba.chunks(4)) {
        println!(".");
        assert_eq!(p1, &p2[..3]);
        assert_eq!(p2[3], 255);
    }
}

#[test]
fn sea_snail() {
    let png_data = include_bytes!("../../resources/sea_snail.png").as_ref();
    let decoder = png::Decoder::new(png_data);
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    let flif_data = include_bytes!("../../resources/sea_snail.flif");
    let data = decode_rgba(flif_data);

    compare_rgb_rgba(&buf, &data)
}

#[test]
fn sea_snail_cutout() {
    let png_data = include_bytes!("../../resources/sea_snail_cutout.png").as_ref();
    let decoder = png::Decoder::new(png_data);
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    let flif_data = include_bytes!("../../resources/sea_snail_cutout.flif");
    let data = decode_rgba(flif_data);

    compare_rgb_rgba(&buf, &data);
}

#[test]
fn flif_logo() {
    let png_data = include_bytes!("../../resources/flif_logo.png").as_ref();
    let decoder = png::Decoder::new(png_data);
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    let flif_data = include_bytes!("../../resources/flif_logo.flif");
    let data = decode_rgba(flif_data);

    assert_eq!(buf, &data[..]);
}

#[test]
fn road() {
    let png_data = include_bytes!("../../resources/road.png").as_ref();
    let decoder = png::Decoder::new(png_data);
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    let flif_data = include_bytes!("../../resources/road.flif");
    let data = decode_gray(flif_data);

    assert_eq!(buf, &data[..]);
}
