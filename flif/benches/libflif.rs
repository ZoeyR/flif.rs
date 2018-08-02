#![cfg(feature = "libflif")]
#![feature(extern_types)]
#![feature(test)]
extern crate test;
extern crate flif_sys as flif;
extern crate libc;

use test::Bencher;

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

#[bench]
fn libflif_cutout_full_decode(b: &mut Bencher) {
    let data = include_bytes!("../../resources/sea_snail_cutout.flif");
    b.iter(|| {
        let raw = decode_rgba(data);
        test::black_box(raw);
    });
}

#[bench]
fn libflif_grey_decode(b: &mut Bencher) {
    let data = include_bytes!("../../resources/road.flif");
    b.iter(|| {
        let raw = decode_gray(data);
        test::black_box(raw);
    });
}
