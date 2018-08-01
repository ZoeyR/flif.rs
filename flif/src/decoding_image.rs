use std::io::Read;

use colors::ColorValue;
use components::transformations::Transform;
pub use error::{Error, Result};
use maniac::{core_pvec, edge_pvec, ManiacTree};
use numbers::rac::Rac;
use numbers::median3;
use numbers::chances::UpdateTable;
use {FlifInfo, Limits};

use pixels::{PixelTrait, ChannelsTrait};
pub use decoder::Decoder;

pub(crate) struct DecodingImage<'a, P: PixelTrait, R: Read + 'a> {
    height: u32,
    width: u32,
    info: &'a FlifInfo,
    rac: &'a mut Rac<R>,
    update_table: &'a UpdateTable,
    limits: &'a Limits,
    data: Vec<P>,
}

#[derive(Debug)]
pub(crate) struct EdgePixelVicinity<P: PixelTrait> {
    pub pixel: P,
    pub chan: P::Channels,
    pub left: Option<ColorValue>,
    pub left2: Option<ColorValue>,
    pub top: Option<ColorValue>,
    pub top2: Option<ColorValue>,
    pub top_left: Option<ColorValue>,
    pub top_right: Option<ColorValue>,
}

#[derive(Debug)]
pub(crate) struct CorePixelVicinity<P: PixelTrait> {
    pub pixel: P,
    pub chan: P::Channels,
    pub left: ColorValue,
    pub left2: ColorValue,
    pub top: ColorValue,
    pub top2: ColorValue,
    pub top_left: ColorValue,
    pub top_right: ColorValue,
}

// safety criterias defined by `debug_assert`s
impl<'a, P: PixelTrait, R: Read> DecodingImage<'a, P, R> {
    pub fn new(
        info: &'a FlifInfo, rac: &'a mut Rac<R>, limits: &'a Limits,
        update_table: &'a UpdateTable,
    ) -> Result<DecodingImage<'a, P, R>> {
        let pixels = (info.header.height * info.header.width) as usize;

        Ok(DecodingImage {
            height: info.header.height,
            width: info.header.width,
            info,
            rac,
            update_table,
            limits,
            data: vec![P::default(); pixels],
        })
    }

    fn check_data(&self) -> bool {
        self.data.len() == (self.width * self.height) as usize
    }

    fn get_idx(&self, x: u32, y: u32) -> usize {
        ((self.width * y) + x) as usize
    }

    unsafe fn get_val(&self, x: u32, y: u32, chan: P::Channels) -> ColorValue {
        debug_assert!(x < self.width && y < self.height && self.check_data());
        self.data.get_unchecked(self.get_idx(x, y)).get_value(chan)
    }

    unsafe fn get_edge_vicinity(&self, x: u32, y: u32, chan: P::Channels)
        -> EdgePixelVicinity<P>
    {
        debug_assert!(x < self.width && y < self.height && self.check_data());
        EdgePixelVicinity {
            pixel: *self.data.get_unchecked(self.get_idx(x, y)),
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

    unsafe fn get_core_vicinity(&self, x: u32, y: u32, chan: P::Channels)
        -> CorePixelVicinity<P>
    {
        debug_assert!(x < self.width - 1 && y < self.height && x > 1 && y > 1 && self.check_data());
        CorePixelVicinity {
            pixel: *self.data.get_unchecked(self.get_idx(x, y)),
            chan,
            top: self.get_val(x, y - 1, chan),
            left: self.get_val(x - 1, y, chan),
            left2: self.get_val(x - 2, y, chan),
            top2: self.get_val(x, y - 2, chan),
            top_left: self.get_val(x - 1, y - 1, chan),
            top_right: self.get_val(x + 1, y - 1, chan),
        }
    }

    unsafe fn process_edge_pixel(
        &mut self,
        x: u32,
        y: u32,
        chan: P::Channels,
        maniac: &mut Option<ManiacTree<'a>>,
    ) -> Result<()>
    {
        let vic = self.get_edge_vicinity(x, y, chan);

        let guess = make_edge_guess(self.info, &vic);
        let c = chan.as_channel();
        let pix = vic.pixel.to_rgba();
        let range = self.info.transform.crange(c, pix);

        let val = if let Some(ref mut maniac) = maniac {
            let snap = self.info.transform.snap(c, pix, guess);
            let pvec = edge_pvec(snap, &vic);
            maniac.process(self.rac, &pvec, snap, range.min, range.max)?
        } else {
            range.min
        };


        let idx = self.get_idx(x, y);
        self.data.get_unchecked_mut(idx).set_value(val, chan);
        Ok(())
    }

    unsafe fn process_core_pixel(
        &mut self,
        x: u32,
        y: u32,
        chan: P::Channels,
        maniac: &mut Option<ManiacTree<'a>>,
    ) -> Result<()>
    {
        let vic = self.get_core_vicinity(x, y, chan);

        let guess = make_core_guess(&vic);
        let c = chan.as_channel();
        let pix = vic.pixel.to_rgba();
        let range = self.info.transform.crange(c, pix);
        let snap = self.info.transform.snap(c, pix, guess);
        let pvec = core_pvec(snap, &vic);

        let val = if let Some(ref mut maniac) = maniac {
            maniac.process(self.rac, &pvec, snap, range.min, range.max)?
        } else {
            range.min
        };


        let idx = self.get_idx(x, y);
        self.data.get_unchecked_mut(idx).set_value(val, chan);
        Ok(())
    }

    pub fn process(&mut self) -> Result<Box<[u8]>> {
        let channels = P::get_chan_order();
        let mut maniac: [Option<ManiacTree>; 4] = Default::default();
        for (i, chan) in channels.as_ref().iter().enumerate() {
            let channel = chan.as_channel();
            let range = self.info.transform.range(channel);
            if range.min == range.max {
                maniac[i] = None;
            } else {
                let tree = ManiacTree::new(
                    self.rac,
                    channel,
                    self.info,
                    self.update_table,
                    self.limits,
                )?;
                maniac[i] = Some(tree);
            }
        }

        for (chan, tree) in channels.as_ref().iter().zip(maniac.iter_mut()) {
            self.channel_pass(*chan, tree)?;
        }

        // undo transofrms and copy raw data
        let n = P::size();
        let mut raw = Vec::with_capacity(n*self.data.len());
        for pixel in self.data.iter_mut() {
            let rgba = self.info.transform.undo(pixel.to_rgba());
            raw.extend(rgba[..n].iter().map(|v| *v as u8 ));
        }

        Ok(raw.into_boxed_slice())
    }

    fn channel_pass(
        &mut self, chan: P::Channels, maniac: &mut Option<ManiacTree<'a>>
    ) -> Result<()> {
        let width = self.width;
        let height = self.height;
        debug_assert!(self.check_data());
        // special case for small images
        if width <= 3 || height <= 2 {
            for y in 0..height {
                for x in 0..width {
                    unsafe { self.process_edge_pixel(x, y, chan, maniac)? }
                }
            }
            return Ok(());
        }
        // process first two rows
        for y in 0..2 {
            for x in 0..width {
                unsafe {
                    self.process_edge_pixel(x, y, chan, maniac)?;
                }
            }
        }
        // main loop
        for y in 2..height {
            // safe because we are sure that x and y inside the image
            unsafe {
                self.process_edge_pixel(0, y, chan, maniac)?;
                self.process_edge_pixel(1, y, chan, maniac)?;
                let end = width - 1;
                for x in 2..end {
                    self.process_core_pixel(x, y, chan, maniac)?;
                }
                self.process_edge_pixel(end, y, chan, maniac)?;
            }
        }
        Ok(())
    }
}


fn make_core_guess<P: PixelTrait>(pix_vic: &CorePixelVicinity<P>) -> i16 {
    let left = pix_vic.left;
    let top = pix_vic.top;
    let top_left = pix_vic.top_left;

    median3(left + top - top_left, left, top)
}

pub(crate) fn make_edge_guess<P>(info: &FlifInfo, vic: &EdgePixelVicinity<P>) -> i16
    where P: PixelTrait, P::Channels: ChannelsTrait
{
    let transformation = &info.transform;

    let left = if let Some(val) = vic.left {
        val
    } else if let Some(val) = vic.top {
        val
    } else if info.second_header.alpha_zero &&
        !vic.chan.is_alpha() && vic.pixel.is_alpha_zero()
    {
        let chan = vic.chan.as_channel();
        (transformation.range(chan).min + transformation.range(chan).max) / 2
    } else {
        transformation.range(vic.chan.as_channel()).min
    };

    let top = if let Some(val) = vic.top {
        val
    } else {
        left
    };

    let top_left = if let Some(val) = vic.top_left {
        val
    } else if let Some(val) = vic.top {
        val
    } else {
        left
    };

    median3(left + top - top_left, left, top)
}
