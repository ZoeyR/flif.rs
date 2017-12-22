use components::transformations::ColorRange;
use super::Transformation;
use ::ColorValue;

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
    fn undo(&self, pixel: &mut [ColorValue]) {
        let red = pixel[1] + pixel[0] + ((1 - pixel[2]) >> 1) - (pixel[1] >> 1);
        let green = pixel[0] - ((-pixel[2])>>1);
        let blue = pixel[0] + ((1 - pixel[2]) >> 1) - (pixel[1] >> 1);

        pixel[0] = red;
        pixel[1] = green;
        pixel[2] = blue;
    }

    fn range(&self, channel: usize) -> ColorRange {
        let (min, max) = match channel {
            0 => (0, self.max),
            1 | 2 => (-self.max, self.max),
            _ => (self.alpha_range.min, self.alpha_range.max),
        };

        ColorRange { min, max }
    }

    fn crange(&self, channel: usize, values: &[ColorValue]) -> ColorRange {
        let origmax4 = (self.max + 1) / 4;

        match channel {
            0 => self.range(0),
            1 => {
                let min = if values[0] < origmax4 - 1 {
                    -3 + (4 * values[0])
                } else if values[0] > (3 * origmax4) - 1 {
                    4 * (values[0] - self.max)
                } else {
                    -self.max
                };

                let max = if values[0] < origmax4 - 1 {
                    3 + (4 * values[0])
                } else if values[0] > (3 * origmax4) - 1 {
                    4 * (self.max - values[0])
                } else {
                    self.max
                };

                ColorRange {min, max}
            },
            2 => {
                let min = if values[0] < origmax4 - 1 {
                    -2 - (2 * values[0])
                } else if values[0] > (3 * origmax4) - 1 {
                    -2 * (self.max - values[0]) + 2 * ((values[1].abs() + 1) / 2)
                } else {
                    ::std::cmp::min(2 * values[0] + 1, (2 * self.max) - (2 * values[0]) - (2 * values[1].abs()) + 1) / 2
                };

                let max = if values[0] < origmax4 - 1 {
                    1 + (2 * values[0]) - (2 * (values[1].abs() / 2))
                } else if values[0] > (3 * origmax4) - 1 {
                    2 * (self.max - values[0])
                } else {
                    ::std::cmp::min(2 * (values[0] - self.max), (-2 * values[0]) - 1 + (2 * (values[1].abs() / 2)))
                };

                ColorRange {min, max}
            },
            n => self.crange(n, values)
        }
    }
}
