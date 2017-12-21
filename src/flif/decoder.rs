use FlifInfo;
use std::io::Read;
use components::header::{Header, SecondHeader};
use error::*;
use {Flif, Metadata};
use numbers::rac::Rac;

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
			second_header
		};

		let mut maniac_vec = Vec::new();
		for channel in 0..main_header.channels as u8 {
			maniac_vec.push(::maniac::ManiacTree::new(&mut rac, channel, &info));
		}

        Ok(Flif {
            header: info.header,
            metadata: info.metadata,
            second_header: info.second_header,
            _image_data: (),
        })
    }
}
