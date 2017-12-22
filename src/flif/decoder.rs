use ColorValue;
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
            let tree = ManiacTree::new(&mut rac, channel, &info)?;
            maniac_vec.push(tree);
        }

        let image_data = non_interlaced_pixels(&info, &mut maniac_vec)?;
        Ok(Flif {
            header: info.header,
            metadata: info.metadata,
            second_header: info.second_header,
            image_data,
        })
    }
}

fn non_interlaced_pixels(info: &FlifInfo, maniac: &mut [ManiacTree]) -> Result<Vec<[ColorValue; 4]>> {
    if info.header.channels != ::components::header::Channels::RGBA {
        Err(Error::Unimplemented(
            "currently decoding only works with RGBA images",
        ))?;
    }

    let channel_order = [3, 0, 1, 2];
    let mut pixels = vec![[0; 4]; (info.header.width * info.header.height) as usize];
    for c in channel_order.iter() {
        for y in 0..info.header.height {
            for x in 0..info.header.width {
                let guess = make_guess(info, &pixels, x, y, *c);
                let snap = info.second_header.transformations.snap(*c, &mut pixels[((info.header.width * y) + x) as usize], guess);
                let pvec = ::maniac::build_pvec(snap, *c, &pixels);
                let value = maniac[*c].process(&pvec, snap);
                pixels[((info.header.width * y) + x) as usize][*c] = value;
            }
        }
    }

    Ok(pixels)
}

fn make_guess(info: &FlifInfo, pixel_data: &[[ColorValue; 4]], x: u32, y: u32, channel: usize) -> i16 {
    let transformation = &info.second_header.transformations;
    let left = if channel < 3 && info.second_header.alpha_zero && x == 0 {
        (transformation.range(channel).min + transformation.range(channel).max) / 2
    } else if x == 0 {
        transformation.range(channel).min
    } else {
        pixel_data[((info.header.width * y) + (x - 1)) as usize][channel] as i16
    };

    let top = if y == 0 {
        left
    } else {
        pixel_data[((info.header.width * (y - 1)) + x) as usize][channel] as i16
    };

    let top_left = if y == 0 {
        left
    } else if x == 0 && y > 0 {
        top
    } else {
        pixel_data[((info.header.width * (y - 1)) + (x - 1)) as usize][channel] as i16
    };

    ((left + top - top_left) + left + top) / 3
}
