use super::Transform;
use components::transformations::ColorRange;
use pixels::Pixel;
use pixels::{ChannelsTrait, Rgba, RgbaChannels};

#[derive(Debug)]
pub struct PermutePlanes {
    max: i16,
}

impl PermutePlanes {
    pub fn new<T: Transform<P>, P: Pixel>(transformation: T) -> PermutePlanes {
        let old_max = P::get_chan_order()
            .as_ref()
            .iter()
            .map(|c| transformation.range(*c).max)
            .max()
            .unwrap();

        let new_max = (((old_max / 4) + 1) * 4) - 1;
        PermutePlanes { max: new_max }
    }
}

impl<P: Pixel> Transform<P> for PermutePlanes {
    fn undo(&self, pixel: P) -> P {
        pixel
    }

    fn range(&self, channel: P::Channels) -> ColorRange {
        let min = match channel.as_channel() {
            RgbaChannels::Red => 0,
            _ => -self.max,
        };

        ColorRange { min, max: self.max }
    }

    fn crange(&self, _channel: P::Channels, _values: P) -> ColorRange {
        unimplemented!()
    }
}
