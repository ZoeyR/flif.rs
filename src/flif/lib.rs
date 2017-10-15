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
