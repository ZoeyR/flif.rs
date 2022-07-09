use super::Transform;
use crate::components::transformations::ColorRange;
use crate::error::*;
use crate::numbers::chances::{ChanceTable, UpdateTable};
use crate::numbers::near_zero::NearZeroCoder;
use crate::numbers::rac::RacRead;
use crate::pixels::{ColorSpace, Rgba, RgbaChannels};

#[derive(Debug)]
pub struct Bounds {
    ranges: [ColorRange; 4],
    previous_transformation: Box<dyn Transform>,
}

impl Bounds {
    pub fn new<R: RacRead>(
        rac: &mut R,
        trans: Box<dyn Transform>,
        channels: ColorSpace,
        update_table: &UpdateTable,
    ) -> Result<Bounds> {
        let mut context = ChanceTable::new(update_table);
        let mut ranges = [ColorRange::default(); 4];
        for &c in &RgbaChannels::ORDER[..channels as usize] {
            let t_range = trans.range(c);
            let c = c as usize;
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
    fn undo(&self, pixel: Rgba) -> Rgba {
        self.previous_transformation.undo(pixel)
    }

    fn range(&self, channel: RgbaChannels) -> ColorRange {
        self.ranges[channel as usize]
    }

    fn crange(&self, channel: RgbaChannels, values: Rgba) -> ColorRange {
        if channel == RgbaChannels::Red || channel == RgbaChannels::Alpha {
            return self.ranges[channel as usize];
        }

        let mut range = self.previous_transformation.crange(channel, values);
        let channel = channel as usize;
        range.min = range.min.max(self.ranges[channel].min);
        range.max = range.max.min(self.ranges[channel].max);

        if range.min > range.max {
            range.min = self.ranges[channel].min;
            range.max = self.ranges[channel].max;
        }
        range
    }
}
