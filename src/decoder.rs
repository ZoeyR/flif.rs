use std::io::Read;

use error::*;
use description::{Channels, Flif, Header};
use numbers::FlifReadExt;

pub struct Decoder<R> {
    reader: R,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Decoder { reader }
    }

    pub fn decode(&mut self) -> Result<Flif> {
        // read the first header
        let _main_header = self.read_main_header()?;

        unimplemented!()
    }

    pub fn read_second_header(&mut self) -> Result<()> {
        unimplemented!()
    }

    pub fn read_metadata(&mut self) -> Result<()> {
        unimplemented!()
    }

    pub fn read_main_header(&mut self) -> Result<Header> {
        // first read in some magic
        let mut magic_buf = [0; 4];
        self.reader.read_exact(&mut magic_buf)?;

        if &magic_buf != b"FLIF" {
            return Err(Error::InvalidHeader {
                desc: "file did not start with magic phrase",
            });
        }

        let flags = self.reader.read_u8()?;

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

        let bytes_per_channel = self.reader.read_u8()? - b'0';
        let width = 1 + self.reader.read_varint()?;
        let height = 1 + self.reader.read_varint()?;

        let num_frames = if animated {
            2 + self.reader.read_varint()?
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
