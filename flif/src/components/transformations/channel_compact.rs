use super::Transform;
use components::transformations::ColorRange;
use error::*;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::near_zero::NearZeroCoder;
use numbers::rac::RacRead;
use pixels::ChannelsTrait;
use pixels::Pixel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelCompact {
    ranges: [ColorRange; 4],
    decompacted: [Vec<i16>; 4],
}
impl ChannelCompact {
    pub fn new<R: RacRead, T: Transform, P: Pixel>(
        rac: &mut R,
        transformation: &T,
        update_table: &UpdateTable,
    ) -> Result<ChannelCompact> {
        let mut context = ChanceTable::new(update_table);
        let mut t = ChannelCompact {
            ranges: Default::default(),
            decompacted: Default::default(),
        };

        for c in P::get_channels().as_ref() {
            let t_range = transformation.range::<P>(*c);
            let c = c.as_channel() as usize;
            t.ranges[c].max = rac.read_near_zero(0, t_range.max - t_range.min, &mut context)?;
            let mut min = t_range.min;
            for i in 0..t.ranges[c].max + 1 {
                t.decompacted[c].push(
                    min + rac.read_near_zero(
                        0,
                        t_range.max - (min + (t.ranges[c].max - i)),
                        &mut context,
                    )?,
                );
                min = t.decompacted[c][i as usize] + 1;
            }
        }

        Ok(t)
    }
}

impl Transform for ChannelCompact {
    fn undo<P: Pixel>(&self, mut pixel: P) -> P {
        for c in P::get_channels().as_ref() {
            let previous = pixel.get_value(*c);
            pixel.set_value(
                self.decompacted[c.as_channel() as usize][previous as usize],
                *c,
            );
        }

        pixel
    }

    fn range<P: Pixel>(&self, channel: P::Channels) -> ColorRange {
        self.ranges[channel.as_channel() as usize]
    }

    fn crange<T: Transform, P: Pixel>(
        &self,
        channel: P::Channels,
        _values: P,
        _previous: &[T],
    ) -> ColorRange {
        self.ranges[channel.as_channel() as usize]
    }
}
