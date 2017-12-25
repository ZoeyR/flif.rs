use components::transformations::ColorRange;
use super::Transform;
use ::ColorValue;

#[derive(Debug)]
pub struct YCoGg {
    max: i16,
    alpha_range: ColorRange,
}

impl YCoGg {
    pub fn new<T: ?Sized + Transform>(transformation: &T) -> YCoGg {
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

impl Transform for YCoGg {
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
                    -3 - (4 * values[0])
                } else if values[0] > (3 * origmax4) - 1 {
                    4 * (values[0] - self.max)
                } else {
                    -self.max
                };

                let max = if values[0] < origmax4 - 1 {
                    3 + (4 * values[0])
                } else if values[0] > (3 * origmax4) - 1 {
                    4*origmax4-4*(1+values[0]-3*origmax4)
                } else {
                    self.max
                };

                ColorRange {min, max}
            },
            2 => {
                let co = values[1];
                let y = values[0];
                let min = if values[0] < origmax4 - 1 {
                    -(2*y+1)
                } else if values[0] > (3 * origmax4) - 1 {
                    -(2*(4*origmax4-1-y)-((1+co.abs())/2)*2)
                } else {
                    -::std::cmp::min(2*origmax4-1+(y-origmax4+1)*2, 2*origmax4+(3*origmax4-1-y)*2-((1+co.abs())/2)*2)
                };

                let max = if values[0] < origmax4 - 1 {
                    1+2*y-(co.abs()/2)*2
                } else if values[0] > (3 * origmax4) - 1 {
                    2*(4*origmax4-1-y)
                } else {
                    -::std::cmp::max(-4*origmax4 + (1+y-2*origmax4)*2, -2*origmax4-(y-origmax4)*2-1+(co.abs()/2)*2)
                };

                ColorRange {min, max}
            },
            n => self.crange(n, values)
        }
    }
}
