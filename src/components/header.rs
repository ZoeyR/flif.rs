#[derive(Debug)]
pub enum Channels {
    Grayscale = 1,
    RGB = 3,
    RGBA = 4,
}

#[derive(Debug)]
pub struct Header {
    pub interlaced: bool,
    pub animated: bool,
    pub channels: Channels,
    pub bytes_per_channel: u8,
    pub width: u32,
    pub height: u32,
    pub num_frames: u32,
}

#[derive(Debug)]
pub struct SecondHeader {
    pub bits_per_pixel: Vec<u8>,
    pub alpha_zero: bool,
    pub loops: u8,
    pub frame_delay: Vec<u16>,
    pub custom_cutoff: bool,
    pub cutoff: Option<u8>,
    pub alpha_divisor: Option<u8>,
    pub custom_bitchance: Option<bool>,
    pub transformations: Vec<()>, // Placeholder until transformations are implemented
    pub invis_pixel_predictor: u8,
}