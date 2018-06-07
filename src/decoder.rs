use super::{Flif, FlifInfo, Metadata, DecodingImage};
use std::io::Read;
use components::header::{BytesPerChannel, Header, SecondHeader};
use numbers::chances::UpdateTable;
use error::*;
use numbers::rac::Rac;
use maniac::ManiacTree;
use colors::{Channel, ChannelSet};

pub struct Decoder<R: Read> {
    info: FlifInfo,
    rac: Rac<R>,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Result<Self> {
        let (info, rac) = identify_internal(reader)?;
        Ok(Decoder { info, rac })
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

        let update_table =
            UpdateTable::new(self.info.second_header.alpha_divisor, self.info.second_header.cutoff);
        let mut maniac_vec: ChannelSet<Option<ManiacTree>> = Default::default();
        for channel in self.info.header.channels {
            let range = self.info.transform.range(channel);
            if range.min == range.max {
                maniac_vec[channel] = None;
            } else {
                let tree = ManiacTree::new(&mut self.rac, channel, &self.info, &update_table)?;
                maniac_vec[channel] = Some(tree);
            }
        }

        let image_data = non_interlaced_pixels(&mut self.rac, &self.info, &mut maniac_vec)?;
        Ok(Flif { info: self.info, image_data })
    }
}

fn identify_internal<R: Read>(mut reader: R) -> Result<(FlifInfo, Rac<R>)> {
    // read the first header
    let main_header = Header::from_reader(&mut reader)?;

    // read the metadata chunks
    let (metadata, non_optional_byte) = Metadata::all_from_reader(&mut reader)?;

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

const CHANNEL_ORDER: [Channel; 4] = [
    Channel::Alpha, Channel::Red, Channel::Green, Channel::Blue
];

fn non_interlaced_pixels<R: Read>(
    rac: &mut Rac<R>,
    info: &FlifInfo,
    maniac: &mut ChannelSet<Option<ManiacTree>>,
) -> Result<DecodingImage> {
    let mut image = DecodingImage::new(info);
    for c in CHANNEL_ORDER.iter().cloned() {
        if info.header.channels.contains_channel(c) {
            for y in 0..info.header.height {
                for x in 0..info.header.width {
                    let guess = make_guess(info, &image, x, y, c);
                    let range = info.transform.crange(c, image.get_vals(y, x));
                    let snap = info.transform.snap(c, image.get_vals(y, x), guess);
                    let pvec = ::maniac::build_pvec(snap, x, y, c, &image);

                    let value = if let Some(ref mut maniac) = maniac[c] {
                        maniac.process(rac, &pvec, snap, range.min, range.max)?
                    } else {
                        range.min
                    };

                    image.set_val(y, x, c, value);
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

fn make_guess(info: &FlifInfo, image: &DecodingImage, x: usize, y: usize, channel: Channel) -> i16 {
    let transformation = &info.transform;
    let left = if x > 0 {
        image.get_val(y, x - 1, channel)
    } else if y > 0 {
        image.get_val(y - 1, x, channel)
    } else if info.second_header.alpha_zero && channel != Channel::Alpha
        && image.get_val(y, x, Channel::Alpha) == 0
    {
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
