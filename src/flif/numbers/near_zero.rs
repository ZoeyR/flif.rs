use std::io::Read;
use error::*;
use num_traits::{PrimInt, Signed};
use numbers::rac::ChanceTable;
use numbers::rac::ChanceTableEntry;
use numbers::rac::IRac;

pub trait NearZeroCoder {
    fn read_near_zero<I: PrimInt + Signed>(
        &mut self,
        min: I,
        max: I,
        context: &mut ChanceTable,
    ) -> Result<I>;

	fn read_near_zero_2<I: PrimInt + Signed>(
        &mut self,
        min: I,
        max: I,
        context: &mut ChanceTable,
    ) -> Result<I>;
}

impl<R> NearZeroCoder for R where R: IRac {
	fn read_near_zero_2<I: PrimInt + Signed>(
        &mut self,
        min: I,
        max: I,
        context: &mut ChanceTable,
    ) -> Result<I> {
		if min > I::zero() {
			Ok(self.read_near_zero(I::zero(), max - min, context)? + min)
		} else if max < I::zero() {
			Ok(self.read_near_zero(min - max, I::zero(), context)? + max)
		} else {
			self.read_near_zero(min, max, context)
		}
	}

    fn read_near_zero<I: PrimInt + Signed>(
        &mut self,
        min: I,
        max: I,
        context: &mut ChanceTable,
    ) -> Result<I> {
        if min > max {
            return Err(Error::Unimplemented("something"));
        }

        if min == max {
            return Ok(min);
        }

        if self.read(context, ChanceTableEntry::Zero)? {
            return Ok(I::zero());
        }

        let sign = if min < I::zero() && max > I::zero() {
            self.read(context, ChanceTableEntry::Sign)?
        } else {
            min >= I::zero()
        };

        let absolute_max = ::std::cmp::max(max, -min);
        let largest_exponent =
            (::std::mem::size_of::<I>() * 8) - absolute_max.leading_zeros() as usize - 1;

        let mut exponent = 0;
        loop {
            if exponent as usize == largest_exponent
                || self.read(context, ChanceTableEntry::Exp(exponent, sign))?
            {
                break;
            }

            exponent += 1;
        }

        // the first mantissa bit is always 1
        let mut have = 1 << exponent;

        // if all other mantissa bits are 1, then the total is have+left
        let mut left = have - 1;

        // read mantissa bits from most-significant to least-significant
        for pos in (0..exponent).rev() {
            left >>= 1;

            // if the bit is 1, then the value will be at least minabs1
            let minabs1 = have | (1 << pos);
            // if the bit is 0, then the value will be at most maxabs0
            let maxabs0 = have | left;
            if I::from(minabs1).unwrap() > absolute_max {
                // 1-bit is impossible (would bump value above maximum),
                // so assume the bit is 0 without reading it
            } else if maxabs0 >= 1 {
                // 0-bit and 1-bit are both possible,
                // so we read the bit and adjust what we have if it is a 1
                if self.read(context, ChanceTableEntry::Mant(pos))? {
                    have = minabs1;
                }
            } else {
                // 0-bit is impossible (would make the value zero),
                // so assume the bit is 1 without reading it
                have = minabs1;
            }
        }
        let have = I::from(have).unwrap();
        Ok(if sign { have } else { -have })
    }
}
