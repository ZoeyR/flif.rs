use std::io::Read;
use error::*;
use num_traits::PrimInt;
use super::rac::Rac;

pub fn read_val<R: Read, T: PrimInt>(rac: &mut Rac<R>, mut min: T, mut max: T) -> Result<T> {
    while max != min {
        let mid = min + ((max - min) >> 1);
        if rac.read_bit()? {
            min = mid + T::one();
        } else {
            max = mid;
        }
    }

    Ok(min)
}

pub fn read_bool<R: Read>(rac: &mut Rac<R>) -> Result<bool> {
    Ok(rac.read_bit()?)
}
