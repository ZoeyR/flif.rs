use ColorValue;
use DecodingImage;
use FlifInfo;
use std::io::Read;
use components::header::{Header, SecondHeader};
use error::*;
use {Flif, Metadata};
use numbers::rac::Rac;
use maniac::ManiacTree;

pub struct Decoder<R> {
    reader: R,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Decoder { reader }
    }

    pub fn decode(&mut self) -> Result<Flif> {
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

        let second_header = SecondHeader::from_rac(&main_header, &mut rac)?;

        let info = FlifInfo {
            header: main_header,
            metadata,
            second_header,
        };

        let mut maniac_vec = Vec::new();
        for channel in 0..main_header.channels as usize {
            let range = info.second_header.transformations.range(channel);
            if range.min == range.max {
                maniac_vec.push(None);
            } else {
                let tree = ManiacTree::new(&mut rac, channel, &info)?;
                maniac_vec.push(Some(tree));
            }
        }

        let image_data = non_interlaced_pixels(&mut rac, &info, &mut maniac_vec)?;
        Ok(Flif {
            info,
            image_data,
        })
    }
}

fn non_interlaced_pixels<R: Read>(rac: &mut Rac<R>, info: &FlifInfo, maniac: &mut [Option<ManiacTree>]) -> Result<DecodingImage> {
    if info.header.channels != ::components::header::Channels::RGBA {
        Err(Error::Unimplemented(
            "currently decoding only works with RGBA images",
        ))?;
    }

    let channel_order = [3, 0, 1, 2];
    let mut image = DecodingImage::new(info);
    for c in channel_order.iter() {
        for y in 0..info.header.height {
            for x in 0..info.header.width {
                let guess = make_guess(info, &image, x, y, *c);
                let range = info.second_header.transformations.crange(*c, image.get_vals(y, x));
                let snap = info.second_header.transformations.snap(*c, image.get_vals(y, x), guess);
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

    for y in 0..info.header.height {
        for x in 0..info.header.width {
            info.second_header.transformations.undo(image.get_vals_mut(y, x));
        }
    }

    Ok(image)
}

fn make_guess(info: &FlifInfo, image: &DecodingImage, x: usize, y: usize, channel: usize) -> i16 {
    let transformation = &info.second_header.transformations;
    let left = if x > 0 {
        image.get_val(y, x - 1, channel)
    } else if y > 0 {
        image.get_val(y - 1, x, channel)
    } else {
        0
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

    ::numbers::median3((left + top - top_left), left, top)
}
