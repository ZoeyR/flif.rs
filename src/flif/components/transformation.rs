#[derive(Copy, Clone, Debug)]
pub enum Transformations {
    ChannelCompact,
    YCoGg,
    PermutePlanes,
    Bounds,
    PalleteAlpha,
    Pallete,
    ColorBuckets,
    DuplicateFrame,
    FrameShape,
    FrameLookback
}

impl Transformations {
    pub fn from_int(num: u32) -> Transformations {
        use self::Transformations::*;
        match num {
            0 => ChannelCompact,
            1 => YCoGg,
            3 => PermutePlanes,
            4 => Bounds,
            5 => PalleteAlpha,
            6 => Pallete,
            7 => ColorBuckets,
            10 => DuplicateFrame,
            11 => FrameShape,
            12 => FrameLookback,
            _ => unimplemented!()
        }
    }
}