extern crate inflate;
extern crate num_traits;
extern crate fnv;

use std::io::Read;

use components::header::{Header, SecondHeader};
use components::metadata::Metadata;
use components::transformations::Transform;
use colors::{Channel, ColorSpace, ColorValue, Pixel};
pub use error::Error;

pub use decoder::Decoder;

mod decoder;
mod numbers;
mod maniac;
pub mod colors;

pub mod components;
mod error;

pub struct Flif {
    info: FlifInfo,
    image_data: DecodingImage,
}

impl Flif {
    pub fn decode<R: Read>(reader: R) -> Result<Self, Error> {
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

        for y in 0..height {
            for x in 0..width {
                let vals = self.image_data.get_vals(y, x);
                for channel in self.info.header.channels {
                    data.push(vals[channel] as u8)
                }
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

struct DecodingImage {
    pub height: usize,
    pub width: usize,
    pub channels: ColorSpace,
    data: Vec<Pixel>,
}

impl DecodingImage {
    pub fn new(info: &FlifInfo) -> DecodingImage {
        DecodingImage {
            height: info.header.height,
            width: info.header.width,
            channels: info.header.channels,
            data: vec![Pixel::default(); info.header.height * info.header.width],
        }
    }

    pub fn get_val(&self, row: usize, col: usize, channel: Channel) -> ColorValue {
        self.data[(self.width * row) + col][channel]
    }

    pub fn set_val(&mut self, row: usize, col: usize, channel: Channel, value: ColorValue) {
        self.data[(self.width * row) + col][channel] = value;
    }

    pub fn get_vals(&self, row: usize, col: usize) -> &Pixel {
        &self.data[(self.width * row) + col]
    }

    pub fn get_vals_mut(&mut self, row: usize, col: usize) -> &mut Pixel {
        &mut self.data[(self.width * row) + col]
    }
}
