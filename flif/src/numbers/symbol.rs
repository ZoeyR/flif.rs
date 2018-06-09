use super::rac::RacRead;
use error::*;
use num_traits::PrimInt;

pub trait UniformSymbolCoder {
    fn read_val<T: PrimInt>(&mut self, min: T, max: T) -> Result<T>;
    fn read_bool(&mut self) -> Result<bool>;
}

impl<R: RacRead> UniformSymbolCoder for R {
    fn read_val<T: PrimInt>(&mut self, mut min: T, mut max: T) -> Result<T> {
        while max != min {
            let mid = min + ((max - min) >> 1);
            if self.read_bit()? {
                min = mid + T::one();
            } else {
                max = mid;
            }
        }

        Ok(min)
    }

    fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_bit()?)
    }
}
