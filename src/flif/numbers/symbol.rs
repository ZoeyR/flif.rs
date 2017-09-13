use std::io::Read;

use num_traits::PrimInt;

use super::rac::{Config, Input};
use error::*;

pub struct UniformSymbolDecoder<'rac, C: 'rac, R: 'rac> {
    rac: &'rac mut Input<C, R>,
}

impl<'rac, C, R> UniformSymbolDecoder<'rac, C, R>
where
    C: Config,
    R: Read,
{
    pub fn new(rac: &'rac mut Input<C, R>) -> Self {
        UniformSymbolDecoder { rac }
    }

    pub fn read_val<T: PrimInt>(&mut self, mut min: T, mut max: T) -> Result<T> {
        while max != min {
            let mid = min + ((max - min) >> 1);
            if self.rac.read_bit()? {
                min = mid + T::one();
            } else {
                max = mid;
            }
        }

        Ok(min)
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.rac.read_bit()?)
    }
}
