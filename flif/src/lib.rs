//! # Example
//! ```rust
//! extern crate flif;
//!
//! use std::fs::File;
//! use std::io::BufReader;
//! use flif::Flif;
//!
//! fn main() {
//!     let file = std::fs::File::open("../resources/flif_logo.flif").unwrap();
//!     // use `BufReader` to improve performance
//!     let reader = BufReader::new(file);
//!     let image = Flif::decode(reader).unwrap();
//!     println!("image info: {:?}", image.info());
//!     let raw_pixels = image.raw();
//! }
//! ```

use std::io::Read;

use components::header::{Header, SecondHeader};
use components::metadata::Metadata;
use components::transformations::Transform;
use decoding_image::DecodingImage;

pub use decoder::Decoder;
pub use error::{Error, Result};

pub mod components;
mod decoder;
mod decoding_image;
mod error;
mod maniac;
mod numbers;
mod pixels;

pub struct Flif {
    info: FlifInfo,
    raw: Box<[u8]>,
}

impl Flif {
    pub fn decode<R: Read>(reader: R) -> Result<Self> {
        Decoder::new(reader)?.decode_image()
    }

    pub fn decode_with_limits<R: Read>(reader: R, limits: Limits) -> Result<Self> {
        Decoder::with_limits(reader, limits)?.decode_image()
    }

    pub fn info(&self) -> &FlifInfo {
        &self.info
    }

    pub fn raw(&self) -> &Box<[u8]> {
        &self.raw
    }

    pub fn into_raw(self) -> Box<[u8]> {
        self.raw
    }
}

/// Limits on input images to prevent OOM based DoS
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Limits {
    /// max size of the compressed metadata in bytes (default: 1 MB)
    pub metadata_chunk: u32,
    /// max number of metadata entries (default: 8)
    pub metadata_count: u32,
    /// max number of pixels: `width * height * frames` (default: 67M = 2<sup>26</sup>)
    pub pixels: u64,
    /// max number of MANIAC nodes (default: 16384 = 2<sup>14</sup>)
    pub maniac_nodes: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            metadata_chunk: 1 << 20,
            metadata_count: 8,
            pixels: 1 << 26,
            maniac_nodes: 1 << 14,
        }
    }
}

#[derive(Debug)]
pub struct FlifInfo {
    pub header: Header,
    pub metadata: Vec<Metadata>,
    pub second_header: SecondHeader,
    transform: Box<dyn Transform>,
}
