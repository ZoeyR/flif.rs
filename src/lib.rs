extern crate inflate;

mod decoder;
mod numbers;
pub mod error;

pub use decoder::Decoder;

pub struct Flif {
    pub header: Header,
    // For this first iteration we won't handle metadata sections. In fact, if they exist we will likely error
    pub metadata: Vec<Metadata>,
    second_header: SecondHeader, //Just like second breakfast
    image_data: () // TODO: decide on format of image data
}

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
struct SecondHeader {
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

pub enum ChunkType {
    Iccp,
    Exif,
    Exmp
}

pub struct Metadata {
    pub chunk_type: ChunkType,
    pub content: Vec<u8>
}

mod private {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_varint_read() {
        use util::ReadVarintExt;

        let buf = [0x82, 0x5F, 0x82, 0x2F];

        let first = buf.as_ref().read_varint().unwrap();
        let second = buf[2..].as_ref().read_varint().unwrap();
        assert_eq!(first, 351);
        assert_eq!(second, 303);
    }

    #[test]
    fn test_varint_overflow_read() {
        use util::ReadVarintExt;

        let buf = [0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        let num = buf.as_ref().read_varint().unwrap();
        assert_eq!(num, 351);
    }
}
