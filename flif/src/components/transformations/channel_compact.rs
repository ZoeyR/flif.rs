use super::Transform;
use crate::components::transformations::ColorRange;
use crate::error::*;
use crate::numbers::chances::{ChanceTable, UpdateTable};
use crate::numbers::near_zero::NearZeroCoder;
use crate::numbers::rac::RacRead;
use crate::pixels::{ColorSpace, Rgba, RgbaChannels};

#[derive(Debug)]
pub struct ChannelCompact {
    ranges: [ColorRange; 4],
    decompacted: [Vec<i16>; 4],
    channels: ColorSpace,
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
            channels,
        };

        for &c in &RgbaChannels::ORDER[..channels as usize] {
            let t_range = transformation.range(c);
            let c = c as usize;
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
    fn undo(&self, mut pixel: Rgba) -> Rgba {
        for &c in &RgbaChannels::ORDER[..self.channels as usize] {
            let c = c as usize;
            pixel.0[c] = self.decompacted[c][pixel.0[c] as usize];
        }

        pixel
    }

    fn range(&self, channel: RgbaChannels) -> ColorRange {
        self.ranges[channel as usize]
    }

    fn crange(&self, channel: RgbaChannels, _values: Rgba) -> ColorRange {
        self.ranges[channel as usize]
    }
}
