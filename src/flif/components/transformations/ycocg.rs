use components::transformations::ColorRange;
use super::Transformation;

#[derive(Debug)]
pub struct YCoGg {
    max: i16,
    alpha_range: ColorRange,
}

impl YCoGg {
    pub fn new<T: ?Sized + Transformation>(transformation: &T) -> YCoGg {
        let max_iter = [
            transformation.range(0).max,
            transformation.range(1).max,
            transformation.range(2).max,
        ];

        let old_max = max_iter.iter().max().unwrap();
        let new_max = (((old_max / 4) + 1) * 4) - 1;
        YCoGg {
            max: new_max,
            alpha_range: transformation.range(3),
        }
    }
}

impl Transformation for YCoGg {
    fn snap(&self, _channel: u8, _values: i16, _pixel: i16) -> i16 {
        unimplemented!()
    }

    fn range(&self, channel: u8) -> ColorRange {
        let (min, max) = match channel {
            0 => (0, self.max),
            1 | 2 => (-self.max, self.max),
            _ => (self.alpha_range.min, self.alpha_range.max),
        };

        ColorRange { min, max }
    }

    fn crange(&self, _channel: u8, _values: i16) -> ColorRange {
        unimplemented!()
    }
}
