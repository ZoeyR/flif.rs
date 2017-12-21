use components::transformations::ColorRange;
use std::io::Read;
use components::header::{Header, SecondHeader};
use error::*;
use numbers::near_zero::NearZeroCoder;
use numbers::rac::ChanceTable;
use numbers::rac::Rac;
use super::Transformation;

#[derive(Debug)]
pub struct Bounds {
    ranges: [ColorRange; 4],
}

impl Bounds {
    pub fn new<R: Read, T: ?Sized + Transformation>(
        rac: &mut Rac<R>,
        trans: &T,
        (ref header, ref second): (&Header, &SecondHeader),
    ) -> Result<Bounds> {
        let mut context = ChanceTable::new(second.alpha_divisor, second.cutoff);
        let mut ranges = [ColorRange { min: 0, max: 0 }; 4];
        for c in 0..header.channels as usize {
            let t_range = trans.range(c as u8);
            ranges[c].min = rac.read_near_zero_2(t_range.min, t_range.max, &mut context)?;
            ranges[c].max = rac.read_near_zero_2(ranges[c].min, t_range.max, &mut context)?;

            // set real min and max
            ranges[c].min = ::std::cmp::max(ranges[c].min, t_range.min);
            ranges[c].max = ::std::cmp::min(ranges[c].max, t_range.max);
        }

        Ok(Bounds { ranges })
    }
}

impl Transformation for Bounds {
    fn snap(&self, _channel: u8, _values: i16, _pixel: i16) -> i16 {
        unimplemented!()
    }

    fn range(&self, channel: u8) -> ColorRange {
        self.ranges[channel as usize]
    }

    fn crange(&self, _channel: u8, _values: i16) -> ColorRange {
        unimplemented!()
    }
}
