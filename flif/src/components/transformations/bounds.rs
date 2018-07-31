use super::Transform;
use colors::{Channel, ChannelSet, ColorSpace};
use components::transformations::ColorRange;
use error::*;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::near_zero::NearZeroCoder;
use numbers::rac::RacRead;

#[derive(Debug)]
pub struct Bounds {
    ranges: ChannelSet<ColorRange>,
    previous_transformation: Box<Transform>,
}

impl Bounds {
    pub fn new<R: RacRead>(
        rac: &mut R,
        trans: Box<Transform>,
        channels: ColorSpace,
        update_table: &UpdateTable,
    ) -> Result<Bounds> {
        let mut context = ChanceTable::new(update_table);
        let mut ranges: ChannelSet<ColorRange> = Default::default();
        for c in channels {
            let t_range = trans.range(c);
            ranges[c].min = rac.read_near_zero(t_range.min, t_range.max, &mut context)?;
            ranges[c].max = rac.read_near_zero(ranges[c].min, t_range.max, &mut context)?;

            // set real min and max
            ranges[c].min = ::std::cmp::max(ranges[c].min, t_range.min);
            ranges[c].max = ::std::cmp::min(ranges[c].max, t_range.max);
        }

        Ok(Bounds {
            ranges,
            previous_transformation: trans,
        })
    }
}

impl Transform for Bounds {
    fn undo(&self, pixel: [i16; 4]) -> [i16; 4] {
        self.previous_transformation.undo(pixel)
    }

    fn range(&self, channel: Channel) -> ColorRange {
        self.ranges[channel]
    }

    fn crange(&self, channel: Channel, values: [i16; 4]) -> ColorRange {
        if channel == Channel::Red || channel == Channel::Alpha {
            return self.ranges[channel];
        }

        let mut range = self.previous_transformation.crange(channel, values);
        range.min = range.min.max(self.ranges[channel].min);
        range.max = range.max.min(self.ranges[channel].max);

        if range.min > range.max {
            range.min = self.ranges[channel].min;
            range.max = self.ranges[channel].max;
        }
        range
    }
}
