use super::Transform;
use components::transformations::ColorRange;
use pixels::Pixel;
use pixels::{ChannelsTrait, RgbaChannels};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermutePlanes {
    max: i16,
}

impl PermutePlanes {
    pub fn new<T: Transform, P: Pixel>(transformation: &T) -> PermutePlanes {
        let old_max = P::get_channels()
            .as_ref()
            .iter()
            .map(|c| transformation.range::<P>(*c).max)
            .max()
            .unwrap();

        let new_max = (((old_max / 4) + 1) * 4) - 1;
        PermutePlanes { max: new_max }
    }
}

impl Transform for PermutePlanes {
    fn undo<P: Pixel>(&self, pixel: P) -> P {
        pixel
    }

    fn range<P: Pixel>(&self, channel: P::Channels) -> ColorRange {
        let min = match channel.as_channel() {
            RgbaChannels::Red => 0,
            _ => -self.max,
        };

        ColorRange { min, max: self.max }
    }

    fn crange<T: Transform, P: Pixel>(
        &self,
        _channel: P::Channels,
        _values: P,
        _previous: &[T],
    ) -> ColorRange {
        unimplemented!()
    }
}
