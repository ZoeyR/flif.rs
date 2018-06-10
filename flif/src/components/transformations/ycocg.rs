use super::Transform;
use colors::{Channel, Pixel};
use components::transformations::ColorRange;

#[derive(Debug)]
pub struct YCoGg {
    max: i16,
    alpha_range: ColorRange,
}

impl YCoGg {
    pub fn new<T: Transform>(transformation: T) -> YCoGg {
        let max_iter = [
            transformation.range(Channel::Red).max,
            transformation.range(Channel::Blue).max,
            transformation.range(Channel::Green).max,
        ];

        let old_max = max_iter.iter().max().unwrap();
        let new_max = (((old_max / 4) + 1) * 4) - 1;
        YCoGg {
            max: new_max,
            alpha_range: transformation.range(Channel::Alpha),
        }
    }
}

impl Transform for YCoGg {
    fn undo(&self, pixel: &mut Pixel) {
        let red = pixel[Channel::Green] + pixel[Channel::Red] + ((1 - pixel[Channel::Blue]) >> 1)
            - (pixel[Channel::Green] >> 1);
        let green = pixel[Channel::Red] - ((-pixel[Channel::Blue]) >> 1);
        let blue =
            pixel[Channel::Red] + ((1 - pixel[Channel::Blue]) >> 1) - (pixel[Channel::Green] >> 1);

        pixel[Channel::Red] = red;
        pixel[Channel::Green] = green;
        pixel[Channel::Blue] = blue;
    }

    fn range(&self, channel: Channel) -> ColorRange {
        let (min, max) = match channel {
            Channel::Red => (0, self.max),
            Channel::Green | Channel::Blue => (-self.max, self.max),
            _ => (self.alpha_range.min, self.alpha_range.max),
        };

        ColorRange { min, max }
    }

    fn crange(&self, channel: Channel, values: &Pixel) -> ColorRange {
        let origmax4 = (self.max + 1) / 4;

        match channel {
            channel @ Channel::Red => self.range(channel),
            Channel::Green => {
                let min = if values[Channel::Red] < origmax4 - 1 {
                    -3 - (4 * values[Channel::Red])
                } else if values[Channel::Red] > (3 * origmax4) - 1 {
                    4 * (values[Channel::Red] - self.max)
                } else {
                    -self.max
                };

                let max = if values[Channel::Red] < origmax4 - 1 {
                    3 + (4 * values[Channel::Red])
                } else if values[Channel::Red] > (3 * origmax4) - 1 {
                    4 * origmax4 - 4 * (1 + values[Channel::Red] - 3 * origmax4)
                } else {
                    self.max
                };

                ColorRange { min, max }
            }
            Channel::Blue => {
                let co = values[Channel::Green];
                let y = values[Channel::Red];
                let min = if values[Channel::Red] < origmax4 - 1 {
                    -(2 * y + 1)
                } else if values[Channel::Red] > (3 * origmax4) - 1 {
                    -(2 * (4 * origmax4 - 1 - y) - ((1 + co.abs()) / 2) * 2)
                } else {
                    -::std::cmp::min(
                        2 * origmax4 - 1 + (y - origmax4 + 1) * 2,
                        2 * origmax4 + (3 * origmax4 - 1 - y) * 2 - ((1 + co.abs()) / 2) * 2,
                    )
                };

                let max = if values[Channel::Red] < origmax4 - 1 {
                    1 + 2 * y - (co.abs() / 2) * 2
                } else if values[Channel::Red] > (3 * origmax4) - 1 {
                    2 * (4 * origmax4 - 1 - y)
                } else {
                    -::std::cmp::max(
                        -4 * origmax4 + (1 + y - 2 * origmax4) * 2,
                        -2 * origmax4 - (y - origmax4) * 2 - 1 + (co.abs() / 2) * 2,
                    )
                };

                ColorRange { min, max }
            }
            Channel::Alpha => self.alpha_range,
        }
    }
}
