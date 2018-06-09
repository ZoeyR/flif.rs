use super::{Flif, FlifInfo, Metadata, DecodingImage, PixelVicinity};
use std::io::Read;
use components::header::{BytesPerChannel, Header, SecondHeader};
use numbers::chances::UpdateTable;
use error::*;
use numbers::rac::Rac;
use numbers::median3;
use maniac::{ManiacTree, build_pvec};
use colors::{Channel, ChannelSet};
use Limits;

pub struct Decoder<R: Read> {
    info: FlifInfo,
    rac: Rac<R>,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Result<Self> {
        let (info, rac) = identify_internal(reader, Default::default())?;
        Ok(Decoder { info, rac })
    }

    pub fn with_limits(reader: R, limits: Limits) -> Result<Self> {
        let (info, rac) = identify_internal(reader, limits)?;
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

fn identify_internal<R: Read>(mut reader: R, limits: Limits)
    -> Result<(FlifInfo, Rac<R>)>
{
    // read the first header
    let main_header = Header::from_reader(&mut reader)?;
    let frames = main_header.num_frames as usize;
    let pixels = main_header.width*main_header.height*frames;
    if pixels > limits.pixels {
        Err(Error::LimitViolation(format!(
            "number of pixels eceeds limit: {} vs {}",
            pixels, limits.pixels,
        )))?
    }

    // read the metadata chunks
    let (metadata, non_optional_byte) = Metadata::all_from_reader(
        &mut reader, &limits
    )?;

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
    for c in CHANNEL_ORDER.iter()
        .filter(|c| info.header.channels.contains_channel(**c)).cloned()
    {
        image.channel_pass(c, |pix_vic| {
            let guess = make_guess(info, &pix_vic);
            let range = info.transform.crange(c, &pix_vic.pixel);
            let snap = info.transform.snap(c, &pix_vic.pixel, guess);
            let pvals = build_pvec(snap, &pix_vic);

            if let Some(ref mut maniac) = maniac[c] {
                maniac.process(rac, &pvals, snap, range.min, range.max)
            } else {
                Ok(range.min)
            }
        })?;
    }

    image.undo_transform(&info.transform);

    Ok(image)
}

fn make_guess(info: &FlifInfo, pix_vic: &PixelVicinity) -> i16 {
    let transformation = &info.transform;
    let chan = pix_vic.chan;
    let left = if let Some(val) = pix_vic.left {
        val
    } else if let Some(val) = pix_vic.top {
        val
    } else if info.second_header.alpha_zero
        && chan != Channel::Alpha
        && pix_vic.pixel[Channel::Alpha] == 0
    {
        (transformation.range(chan).min + transformation.range(chan).max) / 2
    } else {
        transformation.range(chan).min
    };

    let top = if let Some(val) = pix_vic.top { val } else { left };

    let top_left = if let Some(val) = pix_vic.top_left {
        val
    } else if let Some(val) = pix_vic.top {
        val
    } else {
        left
    };

    median3(left + top - top_left, left, top)
}
