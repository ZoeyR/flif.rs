use components::transformations::ColorRange;
use std::io::Read;
use components::header::{Header, SecondHeader};
use error::*;
use numbers::near_zero::NearZeroCoder;
use numbers::rac::ChanceTable;
use numbers::rac::Rac;
use super::Transformation;

#[derive(Debug)]
pub struct ChannelCompact {
    ranges: [ColorRange; 4],
    decompacted: [Vec<i16>; 4],
}
impl ChannelCompact {
    pub fn new<R: Read, T: ?Sized + Transformation>(
        rac: &mut Rac<R>,
        transformation: &T,
        (ref header, ref second): (&Header, &SecondHeader),
    ) -> Result<ChannelCompact> {
        let mut context = ChanceTable::new(second.alpha_divisor, second.cutoff);
        let mut t = ChannelCompact {
            ranges: [ColorRange { min: 0, max: 0 }; 4],
            decompacted: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        };

        for c in 0..header.channels as usize {
            let t_range = transformation.range(c as u8);
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
    fn snap(&self, _channel: u8, _values: i16, _pixel: i16) -> i16 {
        unimplemented!()
    }

    fn range(&self, channel: u8) -> ColorRange {
        self.ranges[channel as usize]
    }

    fn crange(&self, channel: u8, _values: i16) -> ColorRange {
        self.ranges[channel as usize]
    }
}
