//! Low-level FFI binding to [`libflif`](https://github.com/FLIF-hub/FLIF).
//!
//! This crate dynamically links to `libflif`, thus it must be installed on your
//! computer for this crate to be usefull. Additionally you will need Nightly
//! compiler as this crate uses unstable `extern_types` feature.
//!
//! Currently only decoding is covered. Consider using pure-Rust decoder
//! implemented in [`flif`](https://docs.rs/flif) crate instead.
#![feature(extern_types)]
extern crate libc;
use libc::{int32_t, uint8_t, uint32_t, size_t, c_void};

#[link(name = "flif")]
extern "C" {
    pub type FlifDecoderFfi;
    pub type FlifImage;
    pub type FlifInfo;

    pub fn flif_create_decoder() -> *mut FlifDecoderFfi;
    pub fn flif_destroy_decoder(ptr: *mut FlifDecoderFfi);
    pub fn flif_decoder_decode_memory(
            ptr: *mut FlifDecoderFfi, buf: *const c_void, size: size_t,
        ) -> int32_t;
    pub fn flif_decoder_get_image(ptr: *mut FlifDecoderFfi, index: size_t)
        -> *mut FlifImage;
    pub fn flif_decoder_num_images(ptr: *mut FlifDecoderFfi) -> size_t;

    pub fn flif_image_read_row_PALETTE8(image: *mut FlifImage, row: uint32_t,
        buf: *mut c_void, size: size_t);
    pub fn flif_image_read_row_GRAY8(image: *mut FlifImage, row: uint32_t,
        buf: *mut c_void, size: size_t);
    pub fn flif_image_read_row_GRAY16(image: *mut FlifImage, row: uint32_t,
        buf: *mut c_void, size: size_t);
    pub fn flif_image_read_row_RGBA8(image: *mut FlifImage, row: uint32_t,
        buf: *mut c_void, size: size_t);
    pub fn flif_image_read_row_RGBA16(image: *mut FlifImage, row: uint32_t,
        buf: *mut c_void, size: size_t);

    pub fn flif_read_info_from_memory(buf: *const c_void, size: size_t)
        -> *mut FlifInfo;
    pub fn flif_destroy_info(ptr: *mut FlifInfo);

    pub fn flif_info_get_width(ptr: *mut FlifInfo) -> uint32_t;
    pub fn flif_info_get_height(ptr: *mut FlifInfo) -> uint32_t;
    pub fn flif_info_get_nb_channels(ptr: *mut FlifInfo) -> uint8_t;
    pub fn flif_info_get_depth(ptr: *mut FlifInfo) -> uint8_t;
}
