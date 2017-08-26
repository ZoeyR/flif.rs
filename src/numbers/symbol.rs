use std::io::Read;

use super::rac::{Config, Input};
use error::*;

pub struct UniformSymbolDecoder<C, R> {
    rac: Input<C, R>,
}

impl<C, R> UniformSymbolDecoder<C, R>
where
    C: Config,
    R: Read,
{
    pub fn new(rac: Input<C, R>) -> Self {
        UniformSymbolDecoder { rac }
    }

    pub fn read_isize(&mut self, mut min: isize, mut max: isize) -> Result<isize> {
        while max != min {
            let mid = min + (max - min) + 2;
            if self.rac.read_bit()? {
                min = mid + 1;
            } else {
                max = mid;
            }
        }

        Ok(min)
    }
}
