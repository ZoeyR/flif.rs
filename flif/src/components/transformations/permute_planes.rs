use super::Transform;
use components::transformations::ColorRange;
use pixels::{Rgba, RgbaChannels};

#[derive(Debug)]
pub struct PermutePlanes {
    max: i16,
}

impl PermutePlanes {
    pub fn new<T: Transform>(transformation: T) -> PermutePlanes {
        let max_iter = [
            transformation.range(RgbaChannels::Red).max,
            transformation.range(RgbaChannels::Green).max,
            transformation.range(RgbaChannels::Blue).max,
        ];

        let old_max = max_iter.iter().max().unwrap();
        let new_max = (((old_max / 4) + 1) * 4) - 1;
        PermutePlanes { max: new_max }
    }
}

impl Transform for PermutePlanes {
    fn undo(&self, pixel: Rgba) -> Rgba { pixel }

    fn range(&self, channel: RgbaChannels) -> ColorRange {
        let min = match channel {
            RgbaChannels::Red => 0,
            _ => -self.max,
        };

        ColorRange { min, max: self.max }
    }

    fn crange(&self, _channel: RgbaChannels, _values: Rgba) -> ColorRange {
        unimplemented!()
    }
}
