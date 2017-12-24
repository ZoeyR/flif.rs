use std::io::Read;
use error::*;
use numbers::rac::{IRac, Rac};
use numbers::symbol::UniformSymbolCoder;
use numbers::chances::UpdateTable;
use self::channel_compact::ChannelCompact;
use self::bounds::Bounds;
use self::ycocg::YCoGg;
use self::permute_planes::PermutePlanes;
use ColorValue;

mod bounds;
mod channel_compact;
mod ycocg;
mod permute_planes;

pub trait Transformation: ::std::fmt::Debug {
    fn snap(&self, channel: usize, values: &[ColorValue], pixel: ColorValue) -> ColorValue {
        let range = self.crange(channel, values);

        if pixel > range.max {
            range.max
        } else if pixel < range.min {
            range.min
        } else {
            pixel
        }
    }

    fn undo(&self, pixel: &mut [ColorValue]);

    fn range(&self, channel: usize) -> ColorRange;

    fn crange(&self, channel: usize, values: &[ColorValue]) -> ColorRange;
}

#[derive(Debug)]
struct Orig;

impl Transformation for Orig {
    fn undo(&self, _pixel: &mut [ColorValue]) {}

    fn range(&self, _channel: usize) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }

    fn crange(&self, _channel: usize, _values: &[ColorValue]) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }
}

pub(crate) fn load_transformations<R: Read>(
    rac: &mut Rac<R>,
    channels: usize,
    update_table: &UpdateTable,
) -> Result<Box<Transformation>> {
    let mut transformation: Box<Transformation> = Box::new(Orig);
    while rac.read_bit()? {
        let id = rac.read_val(0, 13)?;
        let t = match id {
            0 => Box::new(ChannelCompact::new(
                rac,
                &*transformation,
                channels,
                update_table,
            )?),
            1 => Box::new(YCoGg::new(&*transformation)) as Box<Transformation>,
            3 => Box::new(PermutePlanes::new(&*transformation)) as Box<Transformation>,
            4 => Box::new(Bounds::new(rac, transformation, channels, update_table)?),
            n => {
                println!("found transform: {}", n);
                return Err(Error::Unimplemented(
                    "found unimplemented transformation type",
                ));
            }
        };

        transformation = t;
    }

    Ok(transformation)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorRange {
    pub min: ColorValue,
    pub max: ColorValue,
}
