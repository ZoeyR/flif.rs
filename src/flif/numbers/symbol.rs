use std::io::Read;
use error::*;
use num_traits::PrimInt;
use super::rac::Rac;

pub struct UniformSymbolDecoder<'rac, R: 'rac> {
    rac: &'rac mut Rac<R>,
}

impl<'rac, R: Read> UniformSymbolDecoder<'rac, R> {
    pub fn new(rac: &'rac mut Rac<R>) -> Self {
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
