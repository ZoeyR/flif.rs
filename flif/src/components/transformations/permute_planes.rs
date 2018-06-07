use components::transformations::ColorRange;
use super::Transform;
use colors::{Channel, Pixel};

#[derive(Debug)]
pub struct PermutePlanes {
    max: i16,
}

impl PermutePlanes {
    pub fn new<T: Transform>(transformation: T) -> PermutePlanes {
        let max_iter = [
            transformation.range(Channel::Red).max,
            transformation.range(Channel::Green).max,
            transformation.range(Channel::Blue).max,
        ];

        let old_max = max_iter.iter().max().unwrap();
        let new_max = (((old_max / 4) + 1) * 4) - 1;
        PermutePlanes { max: new_max }
    }
}

impl Transform for PermutePlanes {
    fn undo(&self, _pixel: &mut Pixel) {}

    fn range(&self, channel: Channel) -> ColorRange {
        let min = match channel {
            Channel::Red => 0,
            _ => -self.max,
        };

        ColorRange { min, max: self.max }
    }

    fn crange(&self, _channel: Channel, _values: &Pixel) -> ColorRange {
        unimplemented!()
    }
}
