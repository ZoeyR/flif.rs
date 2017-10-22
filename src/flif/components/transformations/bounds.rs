use std::io::Read;
use components::header::{Header, SecondHeader};
use error::*;
use numbers::near_zero::NearZeroCoder;
use numbers::rac::ChanceTable;
use numbers::rac::Rac;
use super::Transformation;

#[derive(Debug)]
pub struct Bounds {
    min: [i16; 4],
    max: [i16; 4],
}

impl Bounds {
    pub fn new<R: Read, T: ?Sized + Transformation>(
        rac: &mut Rac<R>,
        trans: &T,
        (ref header, ref second): (&Header, &SecondHeader),
    ) -> Result<Bounds> {
        let mut context = ChanceTable::new(second.alpha_divisor, second.cutoff);
        let mut min = [0; 4];
        let mut max = [0; 4];
        for c in 0..header.channels as usize {
            min[c] = rac.read_near_zero(0, trans.max(c as u8) - trans.min(c as u8), &mut context)?
                + trans.min(c as u8);
            max[c] = rac.read_near_zero(0, trans.max(c as u8) - min[c], &mut context)? + min[c];

            // set real min and max
            min[c] = ::std::cmp::max(min[c], trans.min(c as u8));
            max[c] = ::std::cmp::min(max[c], trans.max(c as u8));
        }

        Ok(Bounds { min, max })
    }
}

impl Transformation for Bounds {
    fn snap(&self, _channel: u8, _values: i16, _pixel: i16) -> i16 {
        unimplemented!()
    }

    fn min(&self, channel: u8) -> i16 {
        self.min[channel as usize]
    }

    fn max(&self, channel: u8) -> i16 {
        self.max[channel as usize]
    }

    fn cmin(&self, _channel: u8, _values: i16) -> i16 {
        unimplemented!()
    }

    fn cmax(&self, _channel: u8, _values: i16) -> i16 {
        unimplemented!()
    }
}
