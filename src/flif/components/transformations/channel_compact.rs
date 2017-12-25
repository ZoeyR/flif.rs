use components::transformations::ColorRange;
use error::*;
use numbers::near_zero::NearZeroCoder;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::rac::RacRead;
use super::Transform;
use ColorValue;

#[derive(Debug)]
pub struct ChannelCompact {
    ranges: [ColorRange; 4],
    decompacted: [Vec<i16>; 4],
}
impl ChannelCompact {
    pub fn new<R: RacRead, T: ?Sized + Transform>(
        rac: &mut R,
        transformation: &T,
        channels: usize,
        update_table: &UpdateTable,
    ) -> Result<ChannelCompact> {
        let mut context = ChanceTable::new(update_table);
        let mut t = ChannelCompact {
            ranges: [ColorRange { min: 0, max: 0 }; 4],
            decompacted: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        };

        for c in 0..channels as usize {
            let t_range = transformation.range(c);
            t.ranges[c].max = rac.read_near_zero(0, t_range.max - t_range.min, &mut context)?;
            let mut min = t_range.min;
            for i in 0..t.ranges[c].max {
                t.decompacted[c].push(
                    min
                        + rac.read_near_zero(
                            0,
                            t_range.max - (min + (t.ranges[c].max - i)),
                            &mut context,
                        )?,
                );
                min = t.decompacted[c][i as usize];
            }
        }

        Ok(t)
    }
}

impl Transform for ChannelCompact {
    fn undo(&self, _pixel: &mut [ColorValue]) {}

    fn range(&self, channel: usize) -> ColorRange {
        self.ranges[channel]
    }

    fn crange(&self, channel: usize, _values: &[ColorValue]) -> ColorRange {
        self.ranges[channel]
    }
}
