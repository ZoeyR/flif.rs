use std::io::Read;

use super::{Flif, FlifInfo, Metadata};
use crate::components::header::{BytesPerChannel, Header, SecondHeader};
use crate::decoding_image::DecodingImage;
use crate::error::*;
use crate::numbers::chances::UpdateTable;
use crate::numbers::rac::Rac;
use crate::pixels::ColorSpace;
use crate::pixels::{Greyscale, Rgb, Rgba};
use crate::Limits;

pub struct Decoder<R: Read> {
    limits: Limits,
    info: FlifInfo,
    rac: Rac<R>,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Result<Self> {
        let limits = Default::default();
        let (info, rac) = identify_internal(reader, limits)?;
        Ok(Decoder { limits, info, rac })
    }

    pub fn with_limits(reader: R, limits: Limits) -> Result<Self> {
        let (info, rac) = identify_internal(reader, limits)?;
        Ok(Decoder { limits, info, rac })
    }

    pub fn info(&self) -> &FlifInfo {
        &self.info
    }

    pub fn decode_image(mut self) -> Result<Flif> {
        if self.info.header.interlaced {
            return Err(Error::Unimplemented(
                "Interlaced images are not yet supported.",
            ));
        }

        if self.info.header.num_frames != 1 {
            return Err(Error::Unimplemented(
                "Animated images are not yet supported.",
            ));
        }

        if self.info.header.bytes_per_channel != BytesPerChannel::One {
            return Err(Error::Unimplemented(
                "Only images with 8 bits per channel are supported",
            ));
        }

        if self.info.second_header.custom_bitchance {
            return Err(Error::Unimplemented(
                "Custom bitchances are currently unimplemented in the FLIF standard.",
            ));
        }

        let update_table = UpdateTable::new(
            self.info.second_header.alpha_divisor,
            self.info.second_header.cutoff,
        );

        let raw = match self.info.header.channels {
            ColorSpace::Monochrome => DecodingImage::<Greyscale, _>::new(
                &self.info,
                &mut self.rac,
                &self.limits,
                &update_table,
            )?
            .process()?,
            ColorSpace::RGB => DecodingImage::<Rgb, _>::new(
                &self.info,
                &mut self.rac,
                &self.limits,
                &update_table,
            )?
            .process()?,
            ColorSpace::RGBA => DecodingImage::<Rgba, _>::new(
                &self.info,
                &mut self.rac,
                &self.limits,
                &update_table,
            )?
            .process()?,
        };

        Ok(Flif {
            info: self.info,
            raw,
        })
    }
}

fn identify_internal<R: Read>(mut reader: R, limits: Limits) -> Result<(FlifInfo, Rac<R>)> {
    // read the first header
    let main_header = Header::from_reader(&mut reader, &limits)?;

    // read the metadata chunks
    let (metadata, non_optional_byte) = Metadata::all_from_reader(&mut reader, &limits)?;

    if non_optional_byte != 0 {
        return Err(Error::UnknownRequiredMetadata(non_optional_byte));
    }

    // After this point all values are encoding using the RAC so methods should no longer take
    // the Read object directly.
    let mut rac: Rac<_> = Rac::from_reader(reader)?;

    let (second_header, transform) = SecondHeader::from_rac(&main_header, &mut rac)?;

    Ok((
        FlifInfo {
            header: main_header,
            metadata,
            second_header,
            transform,
        },
        rac,
    ))
}
