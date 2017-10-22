use std::io::Read;
use components::header::{Header, SecondHeader};
use error::*;
use numbers::rac::Rac;
use numbers::symbol::UniformSymbolCoder;
use self::channel_compact::ChannelCompact;
use self::bounds::Bounds;
use self::ycocg::YCoGg;

mod bounds;
mod channel_compact;
mod ycocg;

pub trait Transformation: ::std::fmt::Debug {
    fn snap(&self, channel: u8, values: i16, pixel: i16) -> i16;

    fn min(&self, channel: u8) -> i16;

    fn max(&self, channel: u8) -> i16;

    fn cmin(&self, channel: u8, values: i16) -> i16;

    fn cmax(&self, channel: u8, values: i16) -> i16;
}

#[derive(Debug)]
struct Orig;

impl Transformation for Orig {
    fn snap(&self, _channel: u8, _values: i16, pixel: i16) -> i16 {
        pixel
    }

    fn min(&self, _channel: u8) -> i16 {
        0
    }

    fn max(&self, _channel: u8) -> i16 {
        255
    }

    fn cmin(&self, _channel: u8, _values: i16) -> i16 {
        0
    }

    fn cmax(&self, _channel: u8, _values: i16) -> i16 {
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
            1 => Box::new(YCoGg::new(transforms[transforms.len() - 1].as_ref()))
                as Box<Transformation>,
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
