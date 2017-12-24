use components::transformations::ColorRange;
use std::io::Read;
use error::*;
use numbers::near_zero::NearZeroCoder;
use numbers::rac::ChanceTable;
use numbers::rac::Rac;
use super::Transformation;
use ::ColorValue;

#[derive(Debug)]
pub struct ChannelCompact {
    ranges: [ColorRange; 4],
    decompacted: [Vec<i16>; 4],
}
impl ChannelCompact {
    pub fn new<R: Read, T: ?Sized + Transformation>(
        rac: &mut Rac<R>,
        transformation: &T,
        channels: usize,
        alpha_divisor: u8,
        cutoff: u8,
    ) -> Result<ChannelCompact> {
        let mut context = ChanceTable::new(alpha_divisor, cutoff);
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

impl Transformation for ChannelCompact {
    fn undo(&self, _pixel: &mut [ColorValue]) {

    }

    fn range(&self, channel: usize) -> ColorRange {
        self.ranges[channel]
    }

    fn crange(&self, channel: usize, _values: &[ColorValue]) -> ColorRange {
        self.ranges[channel]
    }
}
