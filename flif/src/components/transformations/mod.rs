use self::bounds::Bounds;
use self::channel_compact::ChannelCompact;
use self::permute_planes::PermutePlanes;
use self::ycocg::YCoGg;
use error::*;
use numbers::chances::UpdateTable;
use numbers::rac::RacRead;
use numbers::symbol::UniformSymbolCoder;
use pixels::{ColorValue, Pixel};

mod bounds;
mod channel_compact;
mod permute_planes;
mod ycocg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transformation {
    Orig,
    ChannelCompact(ChannelCompact),
    YCoGg(YCoGg),
    PermutePlanes(PermutePlanes),
    Bounds(Bounds),
    PaletteAlpha,
    Palette,
    ColorBuckets,
    DuplicateFrame,
    FrameShape,
    FrameLookback,
}

impl Transformation {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Transformation::Orig => "Original (Pseudo Transformation)",
            Transformation::ChannelCompact(_) => "Channel Compact",
            Transformation::YCoGg(_) => "YCoCg",
            Transformation::PermutePlanes(_) => "Permute Planes",
            Transformation::Bounds(_) => "Bounds",
            Transformation::PaletteAlpha => "Palette Alpha",
            Transformation::Palette => "Palette",
            Transformation::ColorBuckets => "Color Buckets",
            Transformation::DuplicateFrame => "Duplicate Frame",
            Transformation::FrameShape => "Frame Shape",
            Transformation::FrameLookback => "Frame Lookback",
        }
    }

    pub(crate) fn from_rac<T: Transform, R: RacRead, P: Pixel>(
        rac: &mut R,
        previous: &T,
        update_table: &UpdateTable,
    ) -> Result<Transformation> {
        let id = rac.read_val(0, 13)?;
        let t = match id {
            0 => Transformation::ChannelCompact(ChannelCompact::new::<R, T, P>(
                rac,
                previous,
                update_table,
            )?),
            1 => Transformation::YCoGg(YCoGg::new::<T, P>(previous)),
            3 => Transformation::PermutePlanes(PermutePlanes::new::<T, P>(previous)),
            4 => Transformation::Bounds(Bounds::new::<T, R, P>(rac, previous, update_table)?),
            5 => Transformation::PaletteAlpha,
            6 => Transformation::Palette,
            7 => Transformation::ColorBuckets,
            10 => Transformation::DuplicateFrame,
            11 => Transformation::FrameShape,
            12 => Transformation::FrameLookback,
            _ => {
                return Err(Error::InvalidOperation(
                    "Invalid transformation identifier read, possibly corrupt file".into(),
                ));
            }
        };

        Ok(t)
    }
}

impl Transform for Transformation {
    #[inline(always)]
    fn snap<P: Pixel>(
        &self,
        channel: P::Channels,
        pixel: P,
        value: ColorValue,
        previous: ColorRange,
    ) -> ColorValue {
        match self {
            Transformation::Orig => Orig.snap(channel, pixel, value, previous),
            Transformation::ChannelCompact(t) => t.snap(channel, pixel, value, previous),
            Transformation::YCoGg(t) => t.snap(channel, pixel, value, previous),
            Transformation::PermutePlanes(t) => t.snap(channel, pixel, value, previous),
            Transformation::Bounds(t) => t.snap(channel, pixel, value, previous),
            Transformation::PaletteAlpha => unimplemented!(),
            Transformation::Palette => unimplemented!(),
            Transformation::ColorBuckets => unimplemented!(),
            Transformation::DuplicateFrame => unimplemented!(),
            Transformation::FrameShape => unimplemented!(),
            Transformation::FrameLookback => unimplemented!(),
        }
    }

    #[inline(always)]
    fn undo<P: Pixel>(&self, pixel: P) -> P {
        match self {
            Transformation::Orig => Orig.undo(pixel),
            Transformation::ChannelCompact(t) => t.undo(pixel),
            Transformation::YCoGg(t) => t.undo(pixel),
            Transformation::PermutePlanes(t) => t.undo(pixel),
            Transformation::Bounds(t) => t.undo(pixel),
            Transformation::PaletteAlpha => unimplemented!(),
            Transformation::Palette => unimplemented!(),
            Transformation::ColorBuckets => unimplemented!(),
            Transformation::DuplicateFrame => unimplemented!(),
            Transformation::FrameShape => unimplemented!(),
            Transformation::FrameLookback => unimplemented!(),
        }
    }

    #[inline(always)]
    fn range<P: Pixel>(&self, channel: P::Channels) -> ColorRange {
        match self {
            Transformation::Orig => Orig.range::<P>(channel),
            Transformation::ChannelCompact(t) => t.range::<P>(channel),
            Transformation::YCoGg(t) => t.range::<P>(channel),
            Transformation::PermutePlanes(t) => t.range::<P>(channel),
            Transformation::Bounds(t) => t.range::<P>(channel),
            Transformation::PaletteAlpha => unimplemented!(),
            Transformation::Palette => unimplemented!(),
            Transformation::ColorBuckets => unimplemented!(),
            Transformation::DuplicateFrame => unimplemented!(),
            Transformation::FrameShape => unimplemented!(),
            Transformation::FrameLookback => unimplemented!(),
        }
    }

    #[inline(always)]
    fn crange<T: Transform, P: Pixel>(
        &self,
        channel: P::Channels,
        values: P,
        previous: &[T],
    ) -> ColorRange {
        match self {
            Transformation::Orig => Orig.crange(channel, values, previous),
            Transformation::ChannelCompact(t) => t.crange(channel, values, previous),
            Transformation::YCoGg(t) => t.crange(channel, values, previous),
            Transformation::PermutePlanes(t) => t.crange(channel, values, previous),
            Transformation::Bounds(t) => t.crange(channel, values, previous),
            Transformation::PaletteAlpha => unimplemented!(),
            Transformation::Palette => unimplemented!(),
            Transformation::ColorBuckets => unimplemented!(),
            Transformation::DuplicateFrame => unimplemented!(),
            Transformation::FrameShape => unimplemented!(),
            Transformation::FrameLookback => unimplemented!(),
        }
    }
}

impl ::std::fmt::Display for Transformation {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

pub trait Transform: ::std::fmt::Debug + Send + Sync {
    fn snap<P: Pixel>(
        &self,
        _channel: P::Channels,
        _pixel: P,
        value: ColorValue,
        range: ColorRange,
    ) -> ColorValue {
        if value > range.max {
            range.max
        } else if value < range.min {
            range.min
        } else {
            value
        }
    }

    fn undo<P: Pixel>(&self, pixel: P) -> P;

    fn range<P: Pixel>(&self, channel: P::Channels) -> ColorRange;

    fn crange<T: Transform, P: Pixel>(
        &self,
        channel: P::Channels,
        values: P,
        previous: &[T],
    ) -> ColorRange;
}

#[derive(Debug)]
struct Orig;

impl Transform for Orig {
    fn undo<P: Pixel>(&self, pixel: P) -> P {
        pixel
    }

    fn range<P: Pixel>(&self, _channel: P::Channels) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }

    fn crange<T: Transform, P: Pixel>(
        &self,
        _channel: P::Channels,
        _values: P,
        _previous: &[T],
    ) -> ColorRange {
        ColorRange { min: 0, max: 255 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformationSet {
    pub set: Vec<Transformation>,
    pub last: Transformation,
}

impl TransformationSet {
    pub fn snap<P: Pixel>(&self, channel: P::Channels, pixel: P, value: ColorValue) -> ColorValue {
        let range = self.last.crange(channel, pixel, &self.set);
        self.last.snap(channel, pixel, value, range)
    }

    pub fn undo<P: Pixel>(&self, mut pixel: P) -> P {
        for t in self.set.iter().rev() {
            pixel = t.undo(pixel);
        }

        self.last.undo(pixel)
    }

    pub fn range<P: Pixel>(&self, channel: P::Channels) -> ColorRange {
        self.last.range::<P>(channel)
    }

    pub fn crange<P: Pixel>(&self, channel: P::Channels, values: P) -> ColorRange {
        self.last.crange(channel, values, &self.set)
    }
}

pub fn load_transformations<R: RacRead, P: 'static + Pixel>(
    rac: &mut R,
    update_table: &UpdateTable,
) -> Result<TransformationSet> {
    let mut transformations = vec![Transformation::Orig];
    while rac.read_bit()? {
        let transformation = Transformation::from_rac::<_, R, P>(
            rac,
            transformations.last().unwrap_or(&Transformation::Orig),
            update_table,
        )?;
        transformations.push(transformation);
    }

    let last = transformations.pop().unwrap();
    Ok(TransformationSet {
        set: transformations,
        last: last,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct ColorRange {
    pub min: ColorValue,
    pub max: ColorValue,
}
