use super::Transform;
use components::transformations::ColorRange;
use error::*;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::near_zero::NearZeroCoder;
use numbers::rac::RacRead;
use pixels::Pixel;
use pixels::{ChannelsTrait, ColorSpace, Rgba, RgbaChannels};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bounds {
    ranges: [ColorRange; 4],
}

impl Bounds {
    pub fn new<T: Transform, R: RacRead, P: Pixel>(
        rac: &mut R,
        trans: &T,
        update_table: &UpdateTable,
    ) -> Result<Bounds> {
        let mut context = ChanceTable::new(update_table);
        let mut ranges = [ColorRange::default(); 4];
        for c in P::get_channels().as_ref() {
            let t_range = trans.range::<P>(*c);
            let c = c.as_channel() as usize;
            ranges[c].min = rac.read_near_zero(t_range.min, t_range.max, &mut context)?;
            ranges[c].max = rac.read_near_zero(ranges[c].min, t_range.max, &mut context)?;

            // set real min and max
            ranges[c].min = ::std::cmp::max(ranges[c].min, t_range.min);
            ranges[c].max = ::std::cmp::min(ranges[c].max, t_range.max);
        }

        Ok(Bounds { ranges })
    }
}

impl Transform for Bounds {
    fn undo<P: Pixel>(&self, pixel: P) -> P {
        pixel
    }

    fn range<P: Pixel>(&self, channel: P::Channels) -> ColorRange {
        self.ranges[channel.as_channel() as usize]
    }

    fn crange<P: Pixel>(
        &self,
        channel: P::Channels,
        values: P,
        previous: ColorRange,
    ) -> ColorRange {
        let rgba_channel = channel.as_channel();
        if rgba_channel == RgbaChannels::Red || rgba_channel == RgbaChannels::Alpha {
            return self.ranges[rgba_channel as usize];
        }

        let mut range = previous;
        let channel = rgba_channel as usize;
        range.min = range.min.max(self.ranges[channel].min);
        range.max = range.max.min(self.ranges[channel].max);

        if range.min > range.max {
            range.min = self.ranges[channel].min;
            range.max = self.ranges[channel].max;
        }
        range
    }
}
