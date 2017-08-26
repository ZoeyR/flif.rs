use std::io::Read;

use inflate::inflate_bytes;

use error::*;
use ::{Channels, Flif, Header, Metadata, ChunkType, SecondHeader};
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

        // read the metadata chunks
        let mut metadata = Vec::new();
        let non_optional_byte = loop {
            let header_first_byte = self.reader.read_u8()?;
            if header_first_byte < 32 {
                break header_first_byte;
            }
            
            metadata.push(self.read_metadata_chunk(header_first_byte)?);
        };

        if non_optional_byte != 0 {
            return Err(Error::UnknownRequiredMetadata(non_optional_byte));
        }


        unimplemented!()
    }

    fn read_second_header(&mut self) -> Result<SecondHeader> {
        unimplemented!()
    }

    /// Reads a metadata section from the image. Since the first byte of the metadata header will have
    /// been read by the main decode function to determine if its optional or not this function will
    /// use the last 3 bytes to determine the metadata type. If in the future this creates a collision
    /// we will have to change the behavior
    fn read_metadata_chunk(&mut self, first_header_byte: u8) -> Result<Metadata> {
        let mut header_buf = [first_header_byte; 4];
        self.reader.read_exact(&mut header_buf[1..])?;
        let chunk_type = match &header_buf {
            b"iCCP" => ChunkType::Iccp,
            b"eXif" => ChunkType::Exif,
            b"eXmp" => ChunkType::Exmp,
            header => return Err(Error::UnknownOptionalMetadata(*header))
        };

        let chunk_size = self.reader.read_varint()?;
        let mut deflated_chunk = vec![0; chunk_size as usize];
        self.reader.read_exact(&mut deflated_chunk)?;
        let inflated_chunk = inflate_bytes(&deflated_chunk).map_err(|s| Error::InvalidMetadata(s))?;

        Ok(Metadata { chunk_type, content: inflated_chunk})
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
