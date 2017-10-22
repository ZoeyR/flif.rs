use super::Transformation;
use numbers::rac::Rac;
use std::io::Read;
use numbers::near_zero::NearZeroCoder;
use components::header::{Header, SecondHeader};
use numbers::rac::ChanceTable;
use error::*;

#[derive(Debug)]
pub struct Bounds {
    min: [u16; 4],
    max: [u16; 4],
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
        }

        Ok(Bounds { min, max })
    }
}

impl Transformation for Bounds {
    fn snap(&self, channel: u8, values: u16, pixel: u16) -> u16 {
        unimplemented!()
    }

    fn min(&self, channel: u8) -> u16 {
        self.min[channel as usize]
    }

    fn max(&self, channel: u8) -> u16 {
        self.max[channel as usize]
    }

    fn cmin(&self, channel: u8, values: u16) -> u16 {
        unimplemented!()
    }

    fn cmax(&self, channel: u8, values: u16) -> u16 {
        unimplemented!()
    }
}
