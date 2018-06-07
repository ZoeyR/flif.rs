extern crate inflate;
extern crate num_traits;
extern crate fnv;

use std::io::Read;

use components::header::{Header, SecondHeader};
use components::metadata::Metadata;
use components::transformations::Transform;
use colors::{Channel, ColorSpace, ColorValue, Pixel};
pub use error::{Error, Result};

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

        for vals in self.image_data.data.iter() {
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

struct DecodingImage {
    pub height: usize,
    pub width: usize,
    pub channels: ColorSpace,
    data: Vec<Pixel>,
}

#[derive(Debug)]
struct PixelVicinity {
    pixel: Pixel,
    chan: Channel,
    is_rgba: bool,

    left: Option<ColorValue>,
    left2: Option<ColorValue>,
    top: Option<ColorValue>,
    top2: Option<ColorValue>,
    top_left: Option<ColorValue>,
    top_right: Option<ColorValue>,
}

// safety criteria for unsafe methods: `x < self.width` and `y < self.height`
// and `self.data.len() == self.width*self.height` must be true
impl DecodingImage {
    pub fn new(info: &FlifInfo) -> DecodingImage {
        DecodingImage {
            height: info.header.height,
            width: info.header.width,
            channels: info.header.channels,
            data: vec![Pixel::default(); info.header.height * info.header.width],
        }
    }

    fn get_idx(&self, x: usize, y: usize) -> usize {
        (self.width * y) + x
    }

    unsafe fn get_val(&self, x: usize, y: usize, chan: Channel) -> ColorValue {
        self.data.get_unchecked(self.get_idx(x, y))[chan]
    }

    unsafe fn get_vicinity(&self, x: usize, y: usize, chan: Channel)
        -> PixelVicinity
    {
        let pixel = *self.data.get_unchecked((self.width * y) + x);
        let is_rgba = self.channels == ColorSpace::RGBA;
        let top = if y != 0 {
            Some(self.get_val(x, y - 1, chan))
        } else {
            None
        };
        let left = if x != 0 {
            Some(self.get_val(x - 1, y, chan))
        } else {
            None
        };
        let left2 = if x > 1 {
            Some(self.get_val(x - 2, y, chan))
        } else {
            None
        };
        let top2 = if y > 1 {
            Some(self.get_val(x, y - 2, chan))
        } else {
            None
        };
        let top_left = if x != 0 && y != 0 {
            Some(self.get_val(x - 1, y - 1, chan))
        } else {
            None
        };
        let top_right = if y != 0 && x + 1 < self.width {
            Some(self.get_val(x + 1, y - 1, chan))
        } else {
            None
        };
        PixelVicinity {
            pixel, chan, is_rgba, left, left2, top, top2, top_left, top_right,
        }
    }

    // iterate over all image pixels and call closure for them without any
    // bound checks
    pub fn channel_pass<F>(&mut self, chan: Channel, mut f: F) -> Result<()>
        where F: FnMut(PixelVicinity) -> Result<ColorValue>
    {
        // strictly speaking it's redundant, but to be safe
        assert_eq!(self.data.len(), self.height*self.width);
        for y in 0..self.height {
            for x in 0..self.width {
                // safe because we are sure that x and y inside the image
                unsafe {
                    let pix_vic = self.get_vicinity(x, y, chan);
                    let val = f(pix_vic)?;
                    let idx = self.get_idx(x, y);
                    self.data.get_unchecked_mut(idx)[chan] = val;
                };
            }
        }
        Ok(())
    }

    pub fn undo_transform(&mut self, transform: &Transform) {
        for vals in self.data.iter_mut() {
            transform.undo(vals);
        }
    }
}
