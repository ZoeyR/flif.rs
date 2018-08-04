use super::Transform;
use components::transformations::ColorRange;
use pixels::Pixel;
use pixels::RgbChannelsTrait;
use pixels::{ChannelsTrait, Rgba, RgbaChannels};

const R: usize = 0;
const G: usize = 1;
const B: usize = 2;

#[derive(Debug)]
pub struct YCoGg<P>
where
    P: Pixel,
    P::Channels: RgbChannelsTrait,
{
    max: i16,
    alpha_range: Option<ColorRange>,
    R: P::Channels,
    G: P::Channels,
    B: P::Channels,
}

impl<P> YCoGg<P>
where
    P: Pixel,
    P::Channels: RgbChannelsTrait,
{
    pub fn new<T: Transform<P>>(transformation: T) -> YCoGg<P> {
        let old_max = P::get_chan_order()
            .as_ref()
            .iter()
            .map(|c| transformation.range(*c).max)
            .max()
            .unwrap();

        let new_max = (((old_max / 4) + 1) * 4) - 1;
        let (alpha_range, A) = if P::is_rgba() {
            let c = P::get_chan_order().as_ref()[0];
            (Some(transformation.range(c)), Some(c))
        } else {
            (None, None)
        };

        YCoGg {
            max: new_max,
            alpha_range,
            R: <P::Channels as RgbChannelsTrait>::red(),
            G: <P::Channels as RgbChannelsTrait>::green(),
            B: <P::Channels as RgbChannelsTrait>::blue(),
        }
    }
}

impl<P> Transform<P> for YCoGg<P>
where
    P: Pixel,
    P::Channels: RgbChannelsTrait,
{
    fn undo(&self, mut pixel: P) -> P {
        let red = pixel.get_value(self.G)
            + pixel.get_value(self.R)
            + ((1 - pixel.get_value(self.B)) >> 1)
            - (pixel.get_value(self.G) >> 1);

        let green = pixel.get_value(self.R) - ((-pixel.get_value(self.B)) >> 1);
        let blue = pixel.get_value(self.R) + ((1 - pixel.get_value(self.B)) >> 1)
            - (pixel.get_value(self.G) >> 1);

        pixel.set_value(red, self.R);
        pixel.set_value(green, self.G);
        pixel.set_value(blue, self.B);

        pixel
    }

    fn range(&self, channel: P::Channels) -> ColorRange {
        let (min, max) = match channel.as_channel() {
            RgbaChannels::Red => (0, self.max),
            RgbaChannels::Green | RgbaChannels::Blue => (-self.max, self.max),
            _ => (self.alpha_range.unwrap().min, self.alpha_range.unwrap().max),
        };

        ColorRange { min, max }
    }

    fn crange(&self, channel: P::Channels, values: P) -> ColorRange {
        let origmax4 = (self.max + 1) / 4;

        match channel.as_channel() {
            RgbaChannels::Red => self.range(channel),
            RgbaChannels::Green => {
                let min = if values.get_value(self.R) < origmax4 - 1 {
                    -3 - (4 * values.get_value(self.R))
                } else if values.get_value(self.R) > (3 * origmax4) - 1 {
                    4 * (values.get_value(self.R) - self.max)
                } else {
                    -self.max
                };

                let max = if values.get_value(self.R) < origmax4 - 1 {
                    3 + (4 * values.get_value(self.R))
                } else if values.get_value(self.R) > (3 * origmax4) - 1 {
                    4 * origmax4 - 4 * (1 + values.get_value(self.R) - 3 * origmax4)
                } else {
                    self.max
                };

                ColorRange { min, max }
            }
            RgbaChannels::Blue => {
                let co = values.get_value(self.G);
                let y = values.get_value(self.R);
                let min = if values.get_value(self.R) < origmax4 - 1 {
                    -(2 * y + 1)
                } else if values.get_value(self.R) > (3 * origmax4) - 1 {
                    -(2 * (4 * origmax4 - 1 - y) - ((1 + co.abs()) / 2) * 2)
                } else {
                    -::std::cmp::min(
                        2 * origmax4 - 1 + (y - origmax4 + 1) * 2,
                        2 * origmax4 + (3 * origmax4 - 1 - y) * 2 - ((1 + co.abs()) / 2) * 2,
                    )
                };

                let max = if values.get_value(self.R) < origmax4 - 1 {
                    1 + 2 * y - (co.abs() / 2) * 2
                } else if values.get_value(self.R) > (3 * origmax4) - 1 {
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
