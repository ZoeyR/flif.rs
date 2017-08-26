use std::io::Read;

use error::*;
use {Flif, Header, Metadata, SecondHeader};
use numbers::FlifReadExt;

pub struct Decoder<R> {
    reader: R,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Decoder { reader }
    }

    pub fn decode(&mut self) -> Result<Flif> {
        // read the first header
        let _main_header = Header::from_reader(&mut self.reader);

        // read the metadata chunks
        let (_metadata, non_optional_byte) = Metadata::all_from_reader(&mut self.reader)?;

        if non_optional_byte != 0 {
            return Err(Error::UnknownRequiredMetadata(non_optional_byte));
        }
        let _ = self.reader.read_u8()?;
        unimplemented!()
    }

    fn read_second_header(&mut self) -> Result<SecondHeader> {
        unimplemented!()
    }
}
