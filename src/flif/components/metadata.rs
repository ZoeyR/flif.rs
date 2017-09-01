use std::io::Read;

use inflate::inflate_bytes;

use error::*;
use numbers::FlifReadExt;

#[derive(Debug)]
pub enum ChunkType {
    Iccp,
    Exif,
    Exmp,
}

enum MetadataType {
    Optional(Metadata),
    Required(u8)
}

#[derive(Debug)]
pub struct Metadata {
    pub chunk_type: ChunkType,
    pub content: Vec<u8>,
}

impl Metadata {

    pub fn all_from_reader<R: Read>(mut reader: R) -> Result<(Vec<Metadata>, u8)> {
        let mut ret = vec![];
        let required_type = loop {
            match Self::from_reader(&mut reader)? {
                MetadataType::Optional(metadata) => ret.push(metadata),
                MetadataType::Required(byte) => break byte
            }
        };

        Ok((ret, required_type))
    }

    /// Reads a metadata section from the image. Since the first byte of the metadata header will have
    /// been read by the main decode function to determine if its optional or not this function will
    /// use the last 3 bytes to determine the metadata type. If in the future this creates a collision
    /// we will have to change the behavior
    fn from_reader<R: Read>(mut reader: R) -> Result<MetadataType> {
        let mut header_buf = [0; 4];

        header_buf[0] = reader.read_u8()?;
        match header_buf[0] {
            byte @ 0...31 => return Ok(MetadataType::Required(byte)),
            _ => {}
        }


        reader.read_exact(&mut header_buf[1..])?;
        let chunk_type = match &header_buf {
            b"iCCP" => ChunkType::Iccp,
            b"eXif" => ChunkType::Exif,
            b"eXmp" => ChunkType::Exmp,
            header => return Err(Error::UnknownOptionalMetadata(*header))
        };

        let chunk_size = reader.read_varint()?;
        let mut deflated_chunk = vec![0; chunk_size as usize];
        reader.read_exact(&mut deflated_chunk)?;
        let inflated_chunk = inflate_bytes(&deflated_chunk).map_err(|s| Error::InvalidMetadata(s))?;

        Ok(MetadataType::Optional(Metadata { chunk_type, content: inflated_chunk}))
    }
}
