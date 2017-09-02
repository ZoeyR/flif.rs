extern crate inflate;
extern crate num_traits;

use components::header::{Header, SecondHeader};
use components::metadata::Metadata;

pub use decoder::Decoder;

mod decoder;
mod numbers;

pub mod components;
pub mod error;


pub struct Flif {
    pub header: Header,
    pub metadata: Vec<Metadata>,
    pub second_header: SecondHeader, //Just like second breakfast
    _image_data: (),                 // TODO: decide on format of image data
}

mod private {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_varint_read() {
        use numbers::FlifReadExt;

        let buf = [0x82, 0x5F, 0x82, 0x2F];

        let first = buf.as_ref().read_varint().unwrap();
        let second = buf[2..].as_ref().read_varint().unwrap();
        assert_eq!(first, 351);
        assert_eq!(second, 303);
    }

    #[test]
    fn test_varint_overflow_read() {
        use numbers::FlifReadExt;

        let buf = [0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        let num = buf.as_ref().read_varint().unwrap();
        assert_eq!(num, u32::max_value());
    }
}
