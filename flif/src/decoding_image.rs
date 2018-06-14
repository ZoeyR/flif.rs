use std::io::Read;

use colors::{Channel, ChannelSet, ColorSpace, ColorValue, Pixel};
use components::transformations::Transform;
pub use error::{Error, Result};
use maniac::ManiacTree;
use numbers::rac::Rac;
use FlifInfo;

pub use decoder::Decoder;

pub(crate) struct DecodingImage {
    height: u32,
    width: u32,
    channels: ColorSpace,
    data: Vec<Pixel>,
}

#[derive(Debug)]
pub(crate) struct EdgePixelVicinity {
    pub pixel: Pixel,
    pub chan: Channel,
    pub is_rgba: bool,

    pub left: Option<ColorValue>,
    pub left2: Option<ColorValue>,
    pub top: Option<ColorValue>,
    pub top2: Option<ColorValue>,
    pub top_left: Option<ColorValue>,
    pub top_right: Option<ColorValue>,
}

#[derive(Debug)]
pub(crate) struct CorePixelVicinity {
    pub pixel: Pixel,
    pub chan: Channel,
    pub is_rgba: bool,

    pub left: ColorValue,
    pub left2: ColorValue,
    pub top: ColorValue,
    pub top2: ColorValue,
    pub top_left: ColorValue,
    pub top_right: ColorValue,
}

type Maniac<'a> = ChannelSet<Option<ManiacTree<'a>>>;

// safety criterias defined by `debug_assert`s
impl DecodingImage {
    pub fn new(info: &FlifInfo) -> DecodingImage {
        let pixels = (info.header.height * info.header.width) as usize;
        DecodingImage {
            height: info.header.height,
            width: info.header.width,
            channels: info.header.channels,
            data: vec![Pixel::default(); pixels],
        }
    }

    fn check_data(&self) -> bool {
        self.data.len() == (self.width * self.height) as usize
    }

    fn get_idx(&self, x: u32, y: u32) -> usize {
        ((self.width * y) + x) as usize
    }

    pub fn get_data(&self) -> &[Pixel] {
        &self.data
    }

    unsafe fn get_val(&self, x: u32, y: u32, chan: Channel) -> ColorValue {
        debug_assert!(x < self.width && y < self.height && self.check_data());
        self.data.get_unchecked(self.get_idx(x, y))[chan]
    }

    unsafe fn get_edge_vicinity(&self, x: u32, y: u32, chan: Channel) -> EdgePixelVicinity {
        debug_assert!(x < self.width && y < self.height && self.check_data());
        EdgePixelVicinity {
            pixel: *self.data.get_unchecked(self.get_idx(x, y)),
            is_rgba: self.channels == ColorSpace::RGBA,
            chan,
            top: if y != 0 {
                Some(self.get_val(x, y - 1, chan))
            } else {
                None
            },
            left: if x != 0 {
                Some(self.get_val(x - 1, y, chan))
            } else {
                None
            },
            left2: if x > 1 {
                Some(self.get_val(x - 2, y, chan))
            } else {
                None
            },
            top2: if y > 1 {
                Some(self.get_val(x, y - 2, chan))
            } else {
                None
            },
            top_left: if x != 0 && y != 0 {
                Some(self.get_val(x - 1, y - 1, chan))
            } else {
                None
            },
            top_right: if y != 0 && x + 1 < self.width {
                Some(self.get_val(x + 1, y - 1, chan))
            } else {
                None
            },
        }
    }

    unsafe fn get_core_vicinity(&self, x: u32, y: u32, chan: Channel) -> CorePixelVicinity {
        debug_assert!(
            x < self.width - 1
                && y < self.height
                && x > 1
                && y > 1
                && self.check_data()
        );
        CorePixelVicinity {
            pixel: *self.data.get_unchecked(self.get_idx(x, y)),
            chan,
            is_rgba: self.channels == ColorSpace::RGBA,
            top: self.get_val(x, y - 1, chan),
            left: self.get_val(x - 1, y, chan),
            left2: self.get_val(x - 2, y, chan),
            top2: self.get_val(x, y - 2, chan),
            top_left: self.get_val(x - 1, y - 1, chan),
            top_right: self.get_val(x + 1, y - 1, chan),
        }
    }

    unsafe fn process_edge_pixel<E, R: Read>(
        &mut self,
        x: u32,
        y: u32,
        chan: Channel,
        maniac: &mut Maniac,
        rac: &mut Rac<R>,
        mut edge_f: E,
    ) -> Result<()>
    where
        E: FnMut(EdgePixelVicinity, &mut Maniac, &mut Rac<R>) -> Result<ColorValue>,
    {
        debug_assert!(x < self.width && y < self.height && self.check_data());
        let pix_vic = self.get_edge_vicinity(x, y, chan);
        let val = edge_f(pix_vic, maniac, rac)?;
        let idx = self.get_idx(x, y);
        self.data.get_unchecked_mut(idx)[chan] = val;
        Ok(())
    }

    // iterate over all image pixels and call closure for them without any
    // bound checks
    pub fn channel_pass<E, F, R: Read>(
        &mut self,
        chan: Channel,
        maniac: &mut Maniac,
        rac: &mut Rac<R>,
        mut edge_f: E,
        mut core_f: F,
    ) -> Result<()>
    where
        E: FnMut(EdgePixelVicinity, &mut Maniac, &mut Rac<R>) -> Result<ColorValue>,
        F: FnMut(CorePixelVicinity, &mut Maniac, &mut Rac<R>) -> Result<ColorValue>,
    {
        let width = self.width;
        let height = self.height;
        debug_assert!(self.check_data());
        // special case for small images
        if width <= 3 || height <= 2 {
            for y in 0..height {
                for x in 0..width {
                    unsafe { self.process_edge_pixel(x, y, chan, maniac, rac, &mut edge_f)? }
                }
            }
            return Ok(());
        }
        // process first two rows
        for y in 0..2 {
            for x in 0..width {
                unsafe {
                    self.process_edge_pixel(x, y, chan, maniac, rac, &mut edge_f)?;
                }
            }
        }
        // main loop
        for y in 2..height {
            // safe because we are sure that x and y inside the image
            unsafe {
                self.process_edge_pixel(0, y, chan, maniac, rac, &mut edge_f)?;
                self.process_edge_pixel(1, y, chan, maniac, rac, &mut edge_f)?;
                let end = width - 1;
                for x in 2..end {
                    let pix_vic = self.get_core_vicinity(x, y, chan);
                    let val = core_f(pix_vic, maniac, rac)?;
                    let idx = self.get_idx(x, y);
                    self.data.get_unchecked_mut(idx)[chan] = val;
                }
                self.process_edge_pixel(end, y, chan, maniac, rac, &mut edge_f)?;
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
