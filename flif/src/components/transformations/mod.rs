use self::bounds::Bounds;
use self::channel_compact::ChannelCompact;
use self::permute_planes::PermutePlanes;
use self::ycocg::YCoGg;
use error::*;
use numbers::chances::UpdateTable;
use numbers::rac::RacRead;
use numbers::symbol::UniformSymbolCoder;
use pixels::RgbChannelsTrait;
use pixels::{ColorSpace, ColorValue, Pixel, Rgba, RgbaChannels};

mod bounds;
mod channel_compact;
mod permute_planes;
mod ycocg;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Transformation {
    ChannelCompact,
    YCoGg,
    PermutePlanes,
    Bounds,
    PaletteAlpha,
    Palette,
    ColorBuckets,
    DuplicateFrame,
    FrameShape,
    FrameLookback,
}

impl Transformation {
    pub(crate) fn from_id(id: u8) -> Option<Transformation> {
        use self::Transformation::*;
        match id {
            0 => Some(ChannelCompact),
            1 => Some(YCoGg),
            3 => Some(PermutePlanes),
            4 => Some(Bounds),
            5 => Some(PaletteAlpha),
            6 => Some(Palette),
            7 => Some(ColorBuckets),
            10 => Some(DuplicateFrame),
            11 => Some(FrameShape),
            12 => Some(FrameLookback),
            _ => None,
        }
    }
}

impl ::std::fmt::Display for Transformation {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::Transformation::*;
        match *self {
            ChannelCompact => write!(f, "Channel Compact"),
            YCoGg => write!(f, "YCoCg"),
            PermutePlanes => write!(f, "Permute Planes"),
            Bounds => write!(f, "Bounds"),
            PaletteAlpha => write!(f, "Palette Alpha"),
            Palette => write!(f, "Palette"),
            ColorBuckets => write!(f, "Color Buckets"),
            DuplicateFrame => write!(f, "Duplicate Frame"),
            FrameShape => write!(f, "Frame Shape"),
            FrameLookback => write!(f, "Frame Lookback"),
        }
    }
}

pub trait Transform<P: Pixel>: ::std::fmt::Debug + Send + Sync {
    fn snap(&self, channel: P::Channels, pixel: P, value: ColorValue) -> ColorValue {
        let range = self.crange(channel, pixel);

        if value > range.max {
            range.max
        } else if value < range.min {
            range.min
        } else {
            value
        }
    }

    fn undo(&self, pixel: P) -> P;

    fn range(&self, channel: P::Channels) -> ColorRange;

    fn crange(&self, channel: P::Channels, values: P) -> ColorRange;
}

impl<P: Pixel> Transform<P> for Box<Transform<P>> {
    fn undo(&self, pixel: P) -> P {
        (**self).undo(pixel)
    }

    fn range(&self, channel: P::Channels) -> ColorRange {
        (**self).range(channel)
    }

    fn crange(&self, channel: P::Channels, values: P) -> ColorRange {
        (**self).crange(channel, values)
    }
}

#[derive(Debug)]
struct Orig;

impl<P: Pixel> Transform<P> for Orig {
    fn undo(&self, pixel: P) -> P {
        pixel
    }

    fn range(&self, _channel: P::Channels) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }

    fn crange(&self, _channel: P::Channels, _values: P) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }
}

pub fn load_transformations<R: RacRead, P: 'static + Pixel>(
    rac: &mut R,
    update_table: &UpdateTable,
) -> Result<(Vec<Transformation>, Box<Transform<P>>)> {
    let mut transform: Box<Transform<P>> = Box::new(Orig);
    let mut transformations = Vec::new();
    while rac.read_bit()? {
        let id = Transformation::from_id(rac.read_val(0, 13)?).ok_or(Error::InvalidOperation(
            "Invalid transformation identifier read, possibly corrupt file".into(),
        ))?;
        transform = match id {
            Transformation::ChannelCompact => {
                Box::new(ChannelCompact::new(rac, transform, update_table)?)
            }
            Transformation::YCoGg => {
                return Err(Error::InvalidOperation(format!(
                    "Found invalid transformation {} in color space where it wasn't valid",
                    id
                )));
            }
            Transformation::PermutePlanes => {
                Box::new(PermutePlanes::new(transform)) as Box<Transform<P>>
            }
            Transformation::Bounds => Box::new(Bounds::new(rac, transform, update_table)?),
            _ => {
                return Err(Error::UnimplementedTransformation(id.to_string()));
            }
        };

        transformations.push(id);
    }

    Ok((transformations, transform))
}

pub fn load_rgb_transformations<R, P: 'static>(
    rac: &mut R,
    update_table: &UpdateTable,
) -> Result<(Vec<Transformation>, Box<Transform<P>>)>
where
    R: RacRead,
    P: Pixel,
    P::Channels: RgbChannelsTrait,
{
    let mut transform: Box<Transform<P>> = Box::new(Orig);
    let mut transformations = Vec::new();
    while rac.read_bit()? {
        let id = Transformation::from_id(rac.read_val(0, 13)?).ok_or(Error::InvalidOperation(
            "Invalid transformation identifier read, possibly corrupt file".into(),
        ))?;
        transform = match id {
            Transformation::ChannelCompact => {
                Box::new(ChannelCompact::new(rac, transform, update_table)?)
            }
            Transformation::YCoGg => Box::new(YCoGg::new(transform)) as Box<Transform<P>>,
            Transformation::PermutePlanes => {
                Box::new(PermutePlanes::new(transform)) as Box<Transform<P>>
            }
            Transformation::Bounds => Box::new(Bounds::new(rac, transform, update_table)?),
            _ => {
                return Err(Error::UnimplementedTransformation(id.to_string()));
            }
        };

        transformations.push(id);
    }

    Ok((transformations, transform))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct ColorRange {
    pub min: ColorValue,
    pub max: ColorValue,
}
