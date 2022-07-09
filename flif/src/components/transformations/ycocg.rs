use super::Transform;
use crate::components::transformations::ColorRange;
use crate::pixels::{Rgba, RgbaChannels};

const R: usize = 0;
const G: usize = 1;
const B: usize = 2;

#[derive(Debug)]
pub struct YCoGg {
    max: i16,
    alpha_range: ColorRange,
    previous_transformation: Box<dyn Transform>,
}

impl YCoGg {
    pub fn new(transformation: Box<dyn Transform>) -> YCoGg {
        let max_iter = [
            transformation.range(RgbaChannels::Red).max,
            transformation.range(RgbaChannels::Blue).max,
            transformation.range(RgbaChannels::Green).max,
        ];

        let old_max = max_iter.iter().max().unwrap();
        let new_max = (((old_max / 4) + 1) * 4) - 1;
        YCoGg {
            max: new_max,
            alpha_range: transformation.range(RgbaChannels::Alpha),
            previous_transformation: transformation,
        }
    }
}

impl Transform for YCoGg {
    fn undo(&self, pixel: Rgba) -> Rgba {
        let pixel = pixel.0;
        let red = pixel[G] + pixel[R] + ((1 - pixel[B]) >> 1) - (pixel[G] >> 1);
        let green = pixel[R] - ((-pixel[B]) >> 1);
        let blue = pixel[R] + ((1 - pixel[B]) >> 1) - (pixel[G] >> 1);
        let alpha = pixel[3];

        self.previous_transformation
            .undo(Rgba([red, green, blue, alpha]))
    }

    fn range(&self, channel: RgbaChannels) -> ColorRange {
        let (min, max) = match channel {
            RgbaChannels::Red => (0, self.max),
            RgbaChannels::Green | RgbaChannels::Blue => (-self.max, self.max),
            _ => (self.alpha_range.min, self.alpha_range.max),
        };

        ColorRange { min, max }
    }

    fn crange(&self, channel: RgbaChannels, values: Rgba) -> ColorRange {
        let values = values.0;
        let origmax4 = (self.max + 1) / 4;

        match channel {
            channel @ RgbaChannels::Red => self.range(channel),
            RgbaChannels::Green => {
                let min = if values[R] < origmax4 - 1 {
                    -3 - (4 * values[R])
                } else if values[R] > (3 * origmax4) - 1 {
                    4 * (values[R] - self.max)
                } else {
                    -self.max
                };

                let max = if values[R] < origmax4 - 1 {
                    3 + (4 * values[R])
                } else if values[R] > (3 * origmax4) - 1 {
                    4 * origmax4 - 4 * (1 + values[R] - 3 * origmax4)
                } else {
                    self.max
                };

                ColorRange { min, max }
            }
            RgbaChannels::Blue => {
                let co = values[G];
                let y = values[R];
                let min = if values[R] < origmax4 - 1 {
                    -(2 * y + 1)
                } else if values[R] > (3 * origmax4) - 1 {
                    -(2 * (4 * origmax4 - 1 - y) - ((1 + co.abs()) / 2) * 2)
                } else {
                    -::std::cmp::min(
                        2 * origmax4 - 1 + (y - origmax4 + 1) * 2,
                        2 * origmax4 + (3 * origmax4 - 1 - y) * 2 - ((1 + co.abs()) / 2) * 2,
                    )
                };

                let max = if values[R] < origmax4 - 1 {
                    1 + 2 * y - (co.abs() / 2) * 2
                } else if values[R] > (3 * origmax4) - 1 {
                    2 * (4 * origmax4 - 1 - y)
                } else {
                    -::std::cmp::max(
                        -4 * origmax4 + (1 + y - 2 * origmax4) * 2,
                        -2 * origmax4 - (y - origmax4) * 2 - 1 + (co.abs() / 2) * 2,
                    )
                };

                ColorRange { min, max }
            }
            RgbaChannels::Alpha => self.alpha_range,
        }
    }
}
