use std::io::Read;

use error::*;
use numbers::FlifReadExt;
use numbers::rac::{Config, Input};
use numbers::symbol::UniformSymbolDecoder;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Channels {
    Grayscale = 1,
    RGB = 3,
    RGBA = 4,
}

#[derive(Debug, Clone, Copy)]
pub enum BytesPerChannel {
    Custom = 0,
    One = 1,
    Two = 2,
}

#[derive(Debug, Copy, Clone)]
pub struct Header {
    pub interlaced: bool,
    pub animated: bool,
    pub channels: Channels,
    pub bytes_per_channel: BytesPerChannel,
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
                desc: "file is corrupt or not a FLIF",
            });
        }

        let flags = reader.read_u8()?;

        let (interlaced, animated) = match flags >> 4 & 0x0F {
            flag @ 3...4 => (flag == 4, false),
            flag @ 5...6 => (flag == 6, true),
            _ => {
                return Err(Error::InvalidHeader {
                    desc: "interlacing/animation bits not valid",
                })
            }
        };

        let channels = match flags & 0x0F {
            1 => Channels::Grayscale,
            3 => Channels::RGB,
            4 => Channels::RGBA,
            _ => {
                return Err(Error::InvalidHeader {
                    desc: "invalid number of channels",
                })
            }
        };

        let bytes_per_channel = match reader.read_u8()? - b'0' {
            0 => BytesPerChannel::Custom,
            1 => BytesPerChannel::One,
            2 => BytesPerChannel::Two,
            _ => {
                return Err(Error::InvalidHeader {
                    desc: "bytes per channel was not a valid value",
                })
            }
        };
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
    pub loops: Option<u8>,
    pub frame_delay: Option<Vec<u16>>,
    pub custom_cutoff: bool,
    pub cutoff: u8,
    pub alpha_divisor: u8,
    pub custom_bitchance: bool,
    pub transformations: Vec<()>, // Placeholder until transformations are implemented
    pub invis_pixel_predictor: u8,
}

impl SecondHeader {
    pub fn from_rac<C: Config, R: Read>(
        main_header: &Header,
        rac: &mut Input<C, R>,
    ) -> Result<Self> {
        let mut uni_decoder = UniformSymbolDecoder::new(rac);

        let bits_per_pixel = (0..main_header.channels as u8)
            .map(|_| match main_header.bytes_per_channel {
                BytesPerChannel::One => Ok(8),
                BytesPerChannel::Two => Ok(16),
                BytesPerChannel::Custom => uni_decoder.read_val(1, 16),
            })
            .collect::<Result<Vec<_>>>()?;

        let alpha_zero = if main_header.channels == Channels::RGBA {
            uni_decoder.read_bool()?
        } else {
            false
        };

        let loops = if main_header.animated {
            Some(uni_decoder.read_val(0, 100)?)
        } else {
            None
        };

        let frame_delay = if main_header.animated {
            Some((0..main_header.num_frames)
                .map(|_| uni_decoder.read_val(0, 60_000))
                .collect::<Result<Vec<_>>>()?)
        } else {
            None
        };

        let custom_cutoff = uni_decoder.read_bool()?;

        let (cutoff, alpha_divisor, custom_bitchance) = if custom_cutoff {
            (
                uni_decoder.read_val(1, 128)?,
                uni_decoder.read_val(2, 128)?,
                uni_decoder.read_bool()?,
            )
        } else {
            (2, 19, false)
        };

        if custom_bitchance {
            // this is currently unimplemented in the reference implementation
            return Err(Error::Unimplemented(
                "custom bitchances are currently unimplemented in the flif standard",
            ));
        }

        // TODO: read transformations
        let invis_pixel_predictor = if uni_decoder.read_bool()? {
            255 // placeholder
        } else {
            uni_decoder.read_val(0, 2)?
        };

        Ok(SecondHeader {
            bits_per_pixel,
            alpha_zero,
            loops,
            frame_delay,
            custom_cutoff,
            cutoff,
            alpha_divisor,
            custom_bitchance,
            transformations: vec![],
            invis_pixel_predictor,
        })
    }
}
