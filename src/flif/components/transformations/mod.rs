use numbers::rac::Rac;
use std::io::Read;
use numbers::symbol::UniformSymbolCoder;
use components::header::{Header, SecondHeader};
use self::channel_compact::ChannelCompact;
use error::*;
use self::bounds::Bounds;
use self::ycocg::YCoGg;

mod bounds;
mod channel_compact;
mod permute_planes;
mod ycocg;

pub trait Transformation: ::std::fmt::Debug {
    fn snap(&self, channel: u8, values: u16, pixel: u16) -> u16;

    fn min(&self, channel: u8) -> u16;

    fn max(&self, channel: u8) -> u16;

    fn cmin(&self, channel: u8, values: u16) -> u16;

    fn cmax(&self, channel: u8, values: u16) -> u16;
}

#[derive(Debug)]
struct Orig;

impl Transformation for Orig {
    fn snap(&self, _channel: u8, _values: u16, pixel: u16) -> u16 {
        pixel
    }

    fn min(&self, _channel: u8) -> u16 {
        0
    }

    fn max(&self, _channel: u8) -> u16 {
        255
    }

    fn cmin(&self, _channel: u8, _values: u16) -> u16 {
        0
    }

    fn cmax(&self, _channel: u8, _values: u16) -> u16 {
        255
    }
}

pub fn load_transformations<R: Read>(
    rac: &mut Rac<R>,
    (ref header, ref second): (&Header, &SecondHeader),
) -> Result<Vec<Box<Transformation>>> {
    let mut transforms: Vec<Box<Transformation>> = Vec::new();
    transforms.push(Box::new(Orig));
    while rac.read_bit()? {
        let id = rac.read_val(0, 13)?;
        let t = match id {
            0 => Box::new(ChannelCompact::new(
                rac,
                transforms[transforms.len() - 1].as_ref(),
                (header, second),
            )?),
            1 => Box::new(YCoGg) as Box<Transformation>,
            4 => Box::new(Bounds::new(
                rac,
                transforms[transforms.len() - 1].as_ref(),
                (header, second),
            )?),
            _ => {
                break;
            }
        };
        transforms.push(t);
    }

    Ok(transforms)
}
