use DecodingImage;
use FlifInfo;
use std::io::Read;
use components::header::{BytesPerChannel, Channels, Header, SecondHeader};
use numbers::chances::UpdateTable;
use error::*;
use {Flif, Metadata};
use numbers::rac::Rac;
use maniac::ManiacTree;

pub struct Decoder<R: Read> {
    reader: R,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Decoder { reader }
    }

    pub fn identify(&mut self) -> Result<FlifInfo> {
        self.identify_internal().map(|tuple| tuple.0)
    }

    pub fn decode(&mut self) -> Result<Flif> {
        let (info, mut rac) = self.identify_internal()?;

        if info.header.channels == Channels::Grayscale {
            return Err(Error::Unimplemented(
                "Grayscale colorspace is not supported for decoding yet.",
            ));
        }

        if info.header.interlaced {
            return Err(Error::Unimplemented(
                "Interlaced images are not yet supported.",
            ));
        }

        if info.header.animated {
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
        let mut maniac_vec = Vec::new();
        for channel in 0..info.header.channels as usize {
            let range = info.transform.range(channel);
            if range.min == range.max {
                maniac_vec.push(None);
            } else {
                let tree = ManiacTree::new(&mut rac, channel, &info, &update_table)?;
                maniac_vec.push(Some(tree));
            }
        }

        let image_data = non_interlaced_pixels(&mut rac, &info, &mut maniac_vec)?;
        Ok(Flif { info, image_data })
    }

    fn identify_internal(&mut self) -> Result<(FlifInfo, Rac<&mut R>)> {
        // read the first header
        let main_header = Header::from_reader(&mut self.reader)?;

        // read the metadata chunks
        let (metadata, non_optional_byte) = Metadata::all_from_reader(&mut self.reader)?;

        if non_optional_byte != 0 {
            return Err(Error::UnknownRequiredMetadata(non_optional_byte));
        }

        // After this point all values are encoding using the RAC so methods should no longer take
        // the Read object directly.
        let mut rac: Rac<_> = Rac::from_reader(&mut self.reader)?;

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
}

fn non_interlaced_pixels<R: Read>(
    rac: &mut Rac<R>,
    info: &FlifInfo,
    maniac: &mut [Option<ManiacTree>],
) -> Result<DecodingImage> {
    let channel_order = [3, 0, 1, 2];
    let mut image = DecodingImage::new(info);
    for c in &channel_order {
        if *c < info.header.channels as usize {
            for y in 0..info.header.height {
                for x in 0..info.header.width {
                    let guess = make_guess(info, &image, x, y, *c);
                    let range = info.transform.crange(*c, image.get_vals(y, x));
                    let snap = info.transform.snap(*c, image.get_vals(y, x), guess);
                    let pvec = ::maniac::build_pvec(snap, x, y, *c, &image);

                    let value = if let Some(ref mut maniac) = maniac[*c] {
                        maniac.process(rac, &pvec, snap, range.min, range.max)?
                    } else {
                        range.min
                    };

                    image.set_val(y, x, *c, value);
                }
            }
        }
    }

    for y in 0..info.header.height {
        for x in 0..info.header.width {
            info.transform.undo(image.get_vals_mut(y, x));
        }
    }

    Ok(image)
}

fn make_guess(info: &FlifInfo, image: &DecodingImage, x: usize, y: usize, channel: usize) -> i16 {
    let transformation = &info.transform;
    let left = if x > 0 {
        image.get_val(y, x - 1, channel)
    } else if y > 0 {
        image.get_val(y - 1, x, channel)
    } else if info.second_header.alpha_zero && channel < 3 && image.get_val(y, x, 3) == 0 {
        (transformation.range(channel).min + transformation.range(channel).max) / 2
    } else {
        transformation.range(channel).min
    };

    let top = if y == 0 {
        left
    } else {
        image.get_val(y - 1, x, channel)
    };

    let top_left = if y == 0 {
        left
    } else if x == 0 && y > 0 {
        top
    } else {
        image.get_val(y - 1, x - 1, channel)
    };

    ::numbers::median3(left + top - top_left, left, top)
}
