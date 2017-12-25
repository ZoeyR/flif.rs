use error::*;
use numbers::rac::RacRead;
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
    pub fn from_id(id: u8) -> Option<Transformation> {
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
        match self {
            &ChannelCompact => write!(f, "Channel Compact"),
            &YCoGg => write!(f, "YCoCg"),
            &PermutePlanes => write!(f, "Permute Planes"),
            &Bounds => write!(f, "Bounds"),
            &PaletteAlpha => write!(f, "Palette Alpha"),
            &Palette => write!(f, "Palette"),
            &ColorBuckets => write!(f, "Color Buckets"),
            &DuplicateFrame => write!(f, "Duplicate Frame"),
            &FrameShape => write!(f, "Frame Shape"),
            &FrameLookback => write!(f, "Frame Lookback"),
        }
    }
}

pub trait Transform: ::std::fmt::Debug {
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

impl Transform for Orig {
    fn undo(&self, _pixel: &mut [ColorValue]) {}

    fn range(&self, _channel: usize) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }

    fn crange(&self, _channel: usize, _values: &[ColorValue]) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }
}

pub fn load_transformations<R: RacRead>(
    rac: &mut R,
    channels: usize,
    update_table: &UpdateTable,
) -> Result<(Vec<Transformation>, Box<Transform>)> {
    let mut transform: Box<Transform> = Box::new(Orig);
    let mut transformations =  Vec::new();
    while rac.read_bit()? {
        let id = Transformation::from_id(rac.read_val(0, 13)?);
        let t = match id {
            Some(Transformation::ChannelCompact) => Box::new(ChannelCompact::new(
                rac,
                &*transform,
                channels,
                update_table,
            )?),
            Some(Transformation::YCoGg) => Box::new(YCoGg::new(&*transform)) as Box<Transform>,
            Some(Transformation::PermutePlanes) => Box::new(PermutePlanes::new(&*transform)) as Box<Transform>,
            Some(Transformation::Bounds) => Box::new(Bounds::new(rac, transform, channels, update_table)?),
            Some(_) => {
                return Err(Error::Unimplemented(
                    "found unimplemented transformation type",
                ));
            },
            None => {
                return Err(Error::InvalidOperation("Invalid transformation identifier read, possibly corrupt file".into()));
            }
        };

        // since a None value on the id causes an early return unwrap is safe here
        transformations.push(id.unwrap());
        transform = t;
    }

    Ok((transformations, transform))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorRange {
    pub min: ColorValue,
    pub max: ColorValue,
}
