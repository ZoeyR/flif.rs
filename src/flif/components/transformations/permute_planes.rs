use components::transformations::ColorRange;
use super::Transform;
use ::ColorValue;

#[derive(Debug)]
pub struct PermutePlanes {
    max: i16,
}

impl PermutePlanes {
    pub fn new<T: Transform>(transformation: T) -> PermutePlanes {
        let max_iter = [
            transformation.range(0).max,
            transformation.range(1).max,
            transformation.range(2).max,
        ];

        let old_max = max_iter.iter().max().unwrap();
        let new_max = (((old_max / 4) + 1) * 4) - 1;
        PermutePlanes { max: new_max }
    }
}

impl Transform for PermutePlanes {
    fn undo(&self, _pixel: &mut [ColorValue]) {

    }

    fn range(&self, channel: usize) -> ColorRange {
        let min = match channel {
            0 => 0,
            _ => -self.max,
        };

        ColorRange { min, max: self.max }
    }

    fn crange(&self, _channel: usize, _values: &[ColorValue]) -> ColorRange {
        unimplemented!()
    }
}
