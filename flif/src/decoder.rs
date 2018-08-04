use std::io::Read;

use super::{Flif, FlifInfo, Metadata};
use components::header::{BytesPerChannel, Header, SecondHeader};
use decoding_image::DecodingImage;
use error::*;
use numbers::chances::UpdateTable;
use numbers::rac::Rac;
use pixels::ColorSpace;
use pixels::{Greyscale, Rgb, Rgba};
use Limits;

pub struct Decoder;

impl Decoder {
    pub fn decode<R: Read>(reader: R) -> Result<Flif> {
        let limits = Default::default();
        Self::decode_image(reader, limits)
    }

    pub fn with_limits<R: Read>(reader: R, limits: Limits) -> Result<Flif> {
        Self::decode_image(reader, limits)
    }

    pub fn identify<R: Read>(reader: R) -> Result<FlifInfo> {
        let limits = Default::default();
        Self::decode_image(reader, limits).map(|flif| flif.info)
    }

    pub fn decode_image<R: Read>(mut reader: R, limits: Limits) -> Result<Flif> {
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

        let second_header = SecondHeader::from_rac(&main_header, &mut rac)?;

        let info = FlifInfo {
            header: main_header,
            metadata,
            second_header,
        };

        if info.header.interlaced {
            return Err(Error::Unimplemented(
                "Interlaced images are not yet supported.",
            ));
        }

        if info.header.num_frames != 1 {
            return Err(Error::Unimplemented(
                "Animated images are not yet supported.",
            ));
        }

        if info.header.bytes_per_channel != BytesPerChannel::One {
            return Err(Error::Unimplemented(
                "Only images with 8 bits per channel are supported",
            ));
        }

        if info.second_header.custom_bitchance {
            return Err(Error::Unimplemented(
                "Custom bitchances are currently unimplemented in the FLIF standard.",
            ));
        }

        let update_table =
            UpdateTable::new(info.second_header.alpha_divisor, info.second_header.cutoff);

        let raw = match info.header.channels {
            ColorSpace::Monochrome => {
                let (_, transform) =
                    ::components::transformations::load_transformations(&mut rac, &update_table)?;
                DecodingImage::<Greyscale, _>::new(
                    &info,
                    transform,
                    &mut rac,
                    &limits,
                    &update_table,
                )?.process()?
            }
            ColorSpace::RGB => {
                let (_, transform) = ::components::transformations::load_rgb_transformations(
                    &mut rac,
                    &update_table,
                )?;
                DecodingImage::<Rgb, _>::new(&info, transform, &mut rac, &limits, &update_table)?
                    .process()?
            }
            ColorSpace::RGBA => {
                let (_, transform) = ::components::transformations::load_rgb_transformations(
                    &mut rac,
                    &update_table,
                )?;
                DecodingImage::<Rgba, _>::new(&info, transform, &mut rac, &limits, &update_table)?
                    .process()?
            }
        };

        Ok(Flif { info: info, raw })
    }
}
