use super::Transform;
use components::transformations::ColorRange;
use pixels::Pixel;
use pixels::{ChannelsTrait, RgbaChannels};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YCoGg {
    max: i16,
    alpha_range: Option<ColorRange>,
}

impl YCoGg {
    pub fn new<T: Transform, P: Pixel>(transformation: &T) -> YCoGg {
        let old_max = P::get_channels()
            .as_ref()
            .iter()
            .map(|c| transformation.range::<P>(*c).max)
            .max()
            .unwrap();

        let new_max = (((old_max / 4) + 1) * 4) - 1;
        let alpha_range = if let Some(alpha) = P::Channels::alpha() {
            Some(transformation.range::<P>(alpha))
        } else {
            None
        };

        YCoGg {
            max: new_max,
            alpha_range,
        }
    }
}

impl Transform for YCoGg {
    fn undo<P: Pixel>(&self, mut pixel: P) -> P {
        let r = P::Channels::red().unwrap();
        let g = P::Channels::green().unwrap();
        let b = P::Channels::blue().unwrap();
        let red = pixel.get_value(g) + pixel.get_value(r) + ((1 - pixel.get_value(b)) >> 1)
            - (pixel.get_value(g) >> 1);

        let green = pixel.get_value(r) - ((-pixel.get_value(b)) >> 1);
        let blue = pixel.get_value(r) + ((1 - pixel.get_value(b)) >> 1) - (pixel.get_value(g) >> 1);

        pixel.set_value(red, r);
        pixel.set_value(green, g);
        pixel.set_value(blue, b);

        pixel
    }

    fn range<P: Pixel>(&self, channel: P::Channels) -> ColorRange {
        let (min, max) = match channel.as_channel() {
            RgbaChannels::Red => (0, self.max),
            RgbaChannels::Green | RgbaChannels::Blue => (-self.max, self.max),
            _ => (self.alpha_range.unwrap().min, self.alpha_range.unwrap().max),
        };

        ColorRange { min, max }
    }

    fn crange<T: Transform, P: Pixel>(
        &self,
        channel: P::Channels,
        values: P,
        _previous: &[T],
    ) -> ColorRange {
        let origmax4 = (self.max + 1) / 4;

        match channel.as_channel() {
            RgbaChannels::Red => self.range::<P>(channel),
            RgbaChannels::Green => {
                let r = values.get_value(P::Channels::red().unwrap());
                let min = if r < origmax4 - 1 {
                    -3 - (4 * r)
                } else if r > (3 * origmax4) - 1 {
                    4 * (r - self.max)
                } else {
                    -self.max
                };

                let max = if r < origmax4 - 1 {
                    3 + (4 * r)
                } else if r > (3 * origmax4) - 1 {
                    4 * origmax4 - 4 * (1 + r - 3 * origmax4)
                } else {
                    self.max
                };

                ColorRange { min, max }
            }
            RgbaChannels::Blue => {
                let r = values.get_value(P::Channels::red().unwrap());
                let g = values.get_value(P::Channels::green().unwrap());
                let co = g;
                let y = r;
                let min = if r < origmax4 - 1 {
                    -(2 * y + 1)
                } else if r > (3 * origmax4) - 1 {
                    -(2 * (4 * origmax4 - 1 - y) - ((1 + co.abs()) / 2) * 2)
                } else {
                    -::std::cmp::min(
                        2 * origmax4 - 1 + (y - origmax4 + 1) * 2,
                        2 * origmax4 + (3 * origmax4 - 1 - y) * 2 - ((1 + co.abs()) / 2) * 2,
                    )
                };

                let max = if r < origmax4 - 1 {
                    1 + 2 * y - (co.abs() / 2) * 2
                } else if r > (3 * origmax4) - 1 {
                    2 * (4 * origmax4 - 1 - y)
                } else {
                    -::std::cmp::max(
                        -4 * origmax4 + (1 + y - 2 * origmax4) * 2,
                        -2 * origmax4 - (y - origmax4) * 2 - 1 + (co.abs() / 2) * 2,
                    )
                };

                ColorRange { min, max }
            }
            RgbaChannels::Alpha => self.alpha_range.unwrap(),
        }
    }
}
