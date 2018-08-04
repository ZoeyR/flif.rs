use super::Transform;
use components::transformations::ColorRange;
use pixels::Pixel;
use pixels::RgbChannelsTrait;
use pixels::{ChannelsTrait, Rgba, RgbaChannels};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YCoGg {
    max: i16,
    alpha_range: Option<ColorRange>,
}

impl YCoGg {
    pub fn new<T: Transform, P: Pixel>(transformation: &T) -> YCoGg {
        let old_max = P::get_chan_order()
            .as_ref()
            .iter()
            .map(|c| transformation.range::<P>(*c).max)
            .max()
            .unwrap();

        let new_max = (((old_max / 4) + 1) * 4) - 1;
        let alpha_range = if P::is_rgba() {
            let c = P::get_chan_order().as_ref()[0];
            Some(transformation.range::<P>(c))
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
        let R = P::Channels::red().unwrap();
        let G = P::Channels::green().unwrap();
        let B = P::Channels::blue().unwrap();
        let red = pixel.get_value(G) + pixel.get_value(R) + ((1 - pixel.get_value(B)) >> 1)
            - (pixel.get_value(G) >> 1);

        let green = pixel.get_value(R) - ((-pixel.get_value(B)) >> 1);
        let blue = pixel.get_value(R) + ((1 - pixel.get_value(B)) >> 1) - (pixel.get_value(G) >> 1);

        pixel.set_value(red, R);
        pixel.set_value(green, G);
        pixel.set_value(blue, B);

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

    fn crange<P: Pixel>(
        &self,
        channel: P::Channels,
        values: P,
        _previous: ColorRange,
    ) -> ColorRange {
        let R = P::Channels::red().unwrap();
        let G = P::Channels::green().unwrap();
        let B = P::Channels::blue().unwrap();

        let origmax4 = (self.max + 1) / 4;

        match channel.as_channel() {
            RgbaChannels::Red => self.range::<P>(channel),
            RgbaChannels::Green => {
                let min = if values.get_value(R) < origmax4 - 1 {
                    -3 - (4 * values.get_value(R))
                } else if values.get_value(R) > (3 * origmax4) - 1 {
                    4 * (values.get_value(R) - self.max)
                } else {
                    -self.max
                };

                let max = if values.get_value(R) < origmax4 - 1 {
                    3 + (4 * values.get_value(R))
                } else if values.get_value(R) > (3 * origmax4) - 1 {
                    4 * origmax4 - 4 * (1 + values.get_value(R) - 3 * origmax4)
                } else {
                    self.max
                };

                ColorRange { min, max }
            }
            RgbaChannels::Blue => {
                let co = values.get_value(G);
                let y = values.get_value(R);
                let min = if values.get_value(R) < origmax4 - 1 {
                    -(2 * y + 1)
                } else if values.get_value(R) > (3 * origmax4) - 1 {
                    -(2 * (4 * origmax4 - 1 - y) - ((1 + co.abs()) / 2) * 2)
                } else {
                    -::std::cmp::min(
                        2 * origmax4 - 1 + (y - origmax4 + 1) * 2,
                        2 * origmax4 + (3 * origmax4 - 1 - y) * 2 - ((1 + co.abs()) / 2) * 2,
                    )
                };

                let max = if values.get_value(R) < origmax4 - 1 {
                    1 + 2 * y - (co.abs() / 2) * 2
                } else if values.get_value(R) > (3 * origmax4) - 1 {
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
