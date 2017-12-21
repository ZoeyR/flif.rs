use std::io::Read;
use components::header::{Header, SecondHeader};
use error::*;
use numbers::rac::{IRac, Rac};
use numbers::symbol::UniformSymbolCoder;
use self::channel_compact::ChannelCompact;
use self::bounds::Bounds;
use self::ycocg::YCoGg;
use self::permute_planes::PermutePlanes;

mod bounds;
mod channel_compact;
mod ycocg;
mod permute_planes;

pub trait Transformation: ::std::fmt::Debug {
    fn snap(&self, channel: u8, values: i16, pixel: i16) -> i16;

    fn range(&self, channel: u8) -> ColorRange;

    fn crange(&self, channel: u8, values: i16) -> ColorRange;
}

#[derive(Debug)]
struct Orig;

impl Transformation for Orig {
    fn snap(&self, _channel: u8, _values: i16, pixel: i16) -> i16 {
        pixel
    }

    fn range(&self, _channel: u8) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }

    fn crange(&self, _channel: u8, _values: i16) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }
}

#[derive(Debug)]
pub struct Transformations {
	transforms: Vec<Box<Transformation>>
}

impl Transformations {
	pub fn empty() -> Transformations {
		Transformations {
			transforms: Vec::new()
		}
	}

	pub fn range(&self, c: usize) -> ColorRange {
		self.transforms[self.transforms.len() - 1].range(c as u8)
	}
}

pub fn load_transformations<R: Read>(
    rac: &mut Rac<R>,
    (ref header, ref second): (&Header, &SecondHeader),
) -> Result<Transformations> {
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
            3 => Box::new(PermutePlanes::new(
                transforms[transforms.len() - 1].as_ref(),
            )) as Box<Transformation>,
            4 => Box::new(Bounds::new(
                rac,
                transforms[transforms.len() - 1].as_ref(),
                (header, second),
            )?),
            n => {
                println!("found transform: {}", n);
                return Err(Error::Unimplemented(
                    "found unimplemented transformation type",
                ));
            }
        };
        transforms.push(t);
    }

    Ok(Transformations {transforms})
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorRange {
    pub min: i16,
    pub max: i16,
}
