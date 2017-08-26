use std::io::Read;

use error::*;
use numbers::FlifReadExt;

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

impl Header {
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self> {
        // first read in some magic
        let mut magic_buf = [0; 4];
        reader.read_exact(&mut magic_buf)?;

        if &magic_buf != b"FLIF" {
            return Err(Error::InvalidHeader {
                desc: "file did not start with magic phrase",
            });
        }

        let flags = reader.read_u8()?;

        let (interlaced, animated) = match flags & 0x0F {
            flag @ 3...4 => (flag == 4, false),
            flag @ 5...6 => (flag == 6, true),
            _ => {
                return Err(Error::InvalidHeader {
                    desc: "interlacing/animation bits not valid",
                })
            }
        };

        let channels = match flags >> 4 & 0x0F {
            1 => Channels::Grayscale,
            3 => Channels::RGB,
            4 => Channels::RGBA,
            _ => {
                return Err(Error::InvalidHeader {
                    desc: "invalid number of channels",
                })
            }
        };

        let bytes_per_channel = reader.read_u8()? - b'0';
        let width = 1 + reader.read_varint()?;
        let height = 1 + reader.read_varint()?;

        let num_frames = if animated {
            2 + reader.read_varint()?
        } else {
            1
        };

        Ok(Header {
            animated,
            interlaced,
            channels,
            bytes_per_channel,
            width,
            height,
            num_frames,
        })
    }
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
