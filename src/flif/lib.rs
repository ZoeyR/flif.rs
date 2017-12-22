extern crate inflate;
extern crate num_traits;

use components::header::{Header, SecondHeader, Channels};
use components::metadata::Metadata;

pub use decoder::Decoder;

mod decoder;
mod numbers;
mod maniac;

pub mod components;
pub mod error;

pub struct Flif {
    pub info: FlifInfo,
    image_data: DecodingImage,
}

impl Flif {
    pub fn get_raw_pixels(&self) -> Vec<u8> {
        let mut data = Vec::new();

        for y in 0..self.image_data.height {
            for x in 0..self.image_data.width {
                let vals = self.image_data.get_vals(y, x);
                data.push((vals[0] as u8));
                data.push((vals[1] as u8));
                data.push((vals[2] as u8));
                data.push(255);
            }
        }
        
        data
    }
}

pub struct FlifInfo {
    pub header: Header,
    pub metadata: Vec<Metadata>,
    pub second_header: SecondHeader,
}

type ColorValue = i16;

struct DecodingImage {
    pub height: usize,
    pub width: usize,
    pub channels: Channels,
    data: Vec<[ColorValue; 4]>,
}

impl DecodingImage {
    pub fn new(info: &FlifInfo) -> DecodingImage {
        DecodingImage {
            height: info.header.height,
            width: info.header.width,
            channels: info.header.channels,
            data: vec![[0, 0, 0, 0]; info.header.height * info.header.width],
        }
    }

    pub fn get_val(&self, row: usize, col: usize, channel: usize) -> ColorValue {
        self.data[(self.width * row) + col][channel]
    }

    pub fn set_val(&mut self, row: usize, col: usize, channel: usize, value: ColorValue) {
        self.data[(self.width * row) + col][channel] = value;
    }

    pub fn get_vals(&self, row: usize, col: usize) -> &[ColorValue; 4] {
        &self.data[(self.width * row) + col]
    }

    pub fn get_vals_mut(&mut self, row: usize, col: usize) -> &mut [ColorValue; 4] {
        &mut self.data[(self.width * row) + col]
    }
}
