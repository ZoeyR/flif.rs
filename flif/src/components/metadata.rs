use std::io::Read;

use crate::error::*;
use crate::numbers::FlifReadExt;
use crate::Limits;

use inflate::inflate_bytes;

#[derive(Copy, Clone, Debug)]
pub enum ChunkType {
    Iccp,
    Exif,
    Exmp,
    Unknown([u8; 4]),
}

enum MetadataType {
    Optional(Metadata),
    Required(u8),
}

#[derive(Clone, Debug)]
pub struct Metadata {
    pub chunk_type: ChunkType,
    pub content: Vec<u8>,
}

impl Metadata {
    pub(crate) fn all_from_reader<R: Read>(
        mut reader: R,
        limits: &Limits,
    ) -> Result<(Vec<Metadata>, u8)> {
        let mut ret = Vec::with_capacity(limits.metadata_count as usize);
        let required_type = loop {
            match Self::from_reader(&mut reader, limits)? {
                MetadataType::Optional(metadata) => ret.push(metadata),
                MetadataType::Required(byte) => break byte,
            }
            if ret.len() > limits.metadata_count as usize {
                Err(Error::LimitViolation(format!(
                    "number of metadata entries exceeds limit: {}",
                    limits.metadata_count,
                )))?;
            }
        };

        Ok((ret, required_type))
    }

    /// Reads a metadata section from the image. Since the first byte of the metadata header will have
    /// been read by the main decode function to determine if its optional or not this function will
    /// use the last 3 bytes to determine the metadata type. If in the future this creates a collision
    /// we will have to change the behavior
    fn from_reader<R: Read>(mut reader: R, limits: &Limits) -> Result<MetadataType> {
        let mut header_buf = [0; 4];

        header_buf[0] = reader.read_u8()?;
        match header_buf[0] {
            0 => return Ok(MetadataType::Required(0)),
            byte @ 1..=31 => return Err(Error::UnknownRequiredMetadata(byte)),
            _ => {}
        }

        reader.read_exact(&mut header_buf[1..])?;
        let chunk_type = match &header_buf {
            b"iCCP" => ChunkType::Iccp,
            b"eXif" => ChunkType::Exif,
            b"eXmp" => ChunkType::Exmp,
            header if header[0] >= b'a' && header[0] <= b'z' => ChunkType::Unknown(*header),
            header => return Err(Error::UnknownCriticalMetadata(*header)),
        };

        let chunk_size = reader.read_varint()?;
        if chunk_size > limits.metadata_chunk as usize {
            Err(Error::LimitViolation(format!(
                "requested metadata chunk size exceeds limit: {} vs {}",
                chunk_size, limits.metadata_chunk,
            )))?;
        }
        let mut deflated_chunk = vec![0; chunk_size];
        reader.read_exact(&mut deflated_chunk)?;
        let inflated_chunk = inflate_bytes(&deflated_chunk).map_err(Error::InvalidMetadata)?;

        Ok(MetadataType::Optional(Metadata {
            chunk_type,
            content: inflated_chunk,
        }))
    }
}
