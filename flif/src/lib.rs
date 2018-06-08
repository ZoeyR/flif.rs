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
extern crate inflate;
extern crate num_traits;
extern crate fnv;

use std::io::Read;

use components::header::{Header, SecondHeader};
use components::metadata::Metadata;
use components::transformations::Transform;
use colors::ColorSpace;
pub use error::{Error, Result};

pub use decoder::Decoder;

mod decoder;
mod numbers;
mod maniac;
mod decoding_image;
pub mod colors;

pub mod components;
mod error;

use decoding_image::DecodingImage;

pub struct Flif {
    info: FlifInfo,
    image_data: DecodingImage,
}

impl Flif {
    pub fn decode<R: Read>(reader: R) -> Result<Self> {
        Decoder::new(reader)?.decode_image()
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

#[derive(Debug)]
pub struct FlifInfo {
    pub header: Header,
    pub metadata: Vec<Metadata>,
    pub second_header: SecondHeader,
    transform: Box<Transform>,
}
