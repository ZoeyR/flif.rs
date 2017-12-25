use components::transformations::ColorRange;
use error::*;
use numbers::near_zero::NearZeroCoder;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::rac::RacRead;
use super::Transform;
use ColorValue;

#[derive(Debug)]
pub struct Bounds {
    ranges: [ColorRange; 4],
    previous_transformation: Box<Transform>,
}

impl Bounds {
    pub fn new<R: RacRead>(
        rac: &mut R,
        trans: Box<Transform>,
        channels: usize,
        update_table: &UpdateTable,
    ) -> Result<Bounds> {
        let mut context = ChanceTable::new(update_table);
        let mut ranges = [ColorRange { min: 0, max: 0 }; 4];
        for c in 0..channels as usize {
            let t_range = trans.range(c);
            ranges[c].min = rac.read_near_zero_2(t_range.min, t_range.max, &mut context)?;
            ranges[c].max = rac.read_near_zero_2(ranges[c].min, t_range.max, &mut context)?;

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
    fn undo(&self, pixel: &mut [ColorValue]) {
        self.previous_transformation.undo(pixel);
    }

    fn range(&self, channel: usize) -> ColorRange {
        self.ranges[channel]
    }

    fn crange(&self, channel: usize, values: &[ColorValue]) -> ColorRange {
        if channel == 0 || channel == 3 {
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
