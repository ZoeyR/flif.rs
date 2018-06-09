use super::Transform;
use colors::{Channel, ChannelSet, ColorSpace, Pixel};
use components::transformations::ColorRange;
use error::*;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::near_zero::NearZeroCoder;
use numbers::rac::RacRead;

#[derive(Debug)]
pub struct ChannelCompact {
    ranges: ChannelSet<ColorRange>,
    decompacted: ChannelSet<Vec<i16>>,
}
impl ChannelCompact {
    pub fn new<R: RacRead, T: Transform>(
        rac: &mut R,
        transformation: T,
        channels: ColorSpace,
        update_table: &UpdateTable,
    ) -> Result<ChannelCompact> {
        let mut context = ChanceTable::new(update_table);
        let mut t = ChannelCompact {
            ranges: Default::default(),
            decompacted: Default::default(),
        };

        for c in channels {
            let t_range = transformation.range(c);
            t.ranges[c].max = rac.read_near_zero(0, t_range.max - t_range.min, &mut context)?;
            let mut min = t_range.min;
            for i in 0..t.ranges[c].max {
                t.decompacted[c].push(
                    min + rac.read_near_zero(
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
    fn undo(&self, _pixel: &mut Pixel) {}

    fn range(&self, channel: Channel) -> ColorRange {
        self.ranges[channel]
    }

    fn crange(&self, channel: Channel, _values: &Pixel) -> ColorRange {
        self.ranges[channel]
    }
}
