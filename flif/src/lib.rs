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
//!     let raw_pixels = image.get_raw_pixels();
//! }
//! ```
extern crate fnv;
extern crate inflate;
extern crate num_traits;

use std::io::Read;

use colors::ColorSpace;
use components::header::{Header, SecondHeader};
use components::metadata::Metadata;
use components::transformations::Transform;
use decoding_image::DecodingImage;

pub use decoder::Decoder;
pub use error::{Error, Result};

pub mod colors;
pub mod components;
mod decoder;
mod decoding_image;
mod error;
mod maniac;
mod numbers;

pub struct Flif {
    info: FlifInfo,
    image_data: DecodingImage,
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

    pub fn get_raw_pixels(&self) -> Vec<u8> {
        let n = match self.info.header.channels {
            ColorSpace::RGBA => 4,
            ColorSpace::RGB => 3,
            ColorSpace::Monochrome => 1,
        };
        let width = self.info.header.width;
        let height = self.info.header.height;
        let mut data = Vec::with_capacity(n * width * height);

        for vals in self.image_data.get_data().iter() {
            for channel in self.info.header.channels {
                data.push(vals[channel] as u8)
            }
        }

        data
    }
}

/// Limits on input images to prevent OOM based DoS
#[derive(Copy, Clone, Debug)]
pub struct Limits {
    /// max size of the compressed metadata in bytes (default: 1 MB)
    pub metadata_chunk: usize,
    /// max number of metadata entries (default: 8)
    pub metadata_count: usize,
    /// max number of pixels: `width * height * frames` (default: 2<sup>26</sup>)
    pub pixels: usize,
    /// max number of MANIAC nodes (default: 2<sup>14</sup>)
    pub maniac_nodes: usize,
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
    transform: Box<Transform>,
}
