use error::*;
use num_traits::PrimInt;
use numbers::chances::{ChanceTable, ChanceTableEntry};
use numbers::rac::RacRead;

use std::cmp;

pub trait NearZeroCoder {
    fn read_near_zero<I: PrimInt>(
        &mut self,
        min: I,
        max: I,
        context: &mut ChanceTable,
    ) -> Result<I>;
}

impl<R: RacRead> NearZeroCoder for R {
    fn read_near_zero<I: PrimInt>(
        &mut self,
        min: I,
        max: I,
        context: &mut ChanceTable,
    ) -> Result<I> {
        let delta = cmp::min(max, cmp::max(I::zero(), min));
        let min = min - delta;
        let max = max - delta;
        Ok(read_near_zero_inner(self, min, max, context)? + delta)
    }
}

#[inline(always)]
fn read_near_zero_inner<R: RacRead, I: PrimInt>(
    read: &mut R,
    min: I,
    max: I,
    context: &mut ChanceTable,
) -> Result<I> {
    if min > max {
        return Err(Error::InvalidOperation(
            "near zero integer reading was passed a larger min than max".into(),
        ));
    }

    if min == max {
        return Ok(min);
    }

    if read.read(context, ChanceTableEntry::Zero)? {
        return Ok(I::zero());
    }

    let sign = if min < I::zero() && max > I::zero() {
        read.read(context, ChanceTableEntry::Sign)?
    } else {
        min >= I::zero()
    };

    // bitwise negation is done here since the methods exist for both unsigned and signed numbers,
    // it is safe because the else clause can only be hit when I is signed.
    let absolute_max = if sign { max } else { (!min) + I::one() };

    let largest_exponent =
        (::std::mem::size_of::<I>() * 8) - absolute_max.leading_zeros() as usize - 1;

    let mut exponent = 0;
    loop {
        if exponent as usize == largest_exponent
            || read.read(context, ChanceTableEntry::Exp(exponent, sign))?
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
            if read.read(context, ChanceTableEntry::Mant(pos))? {
                have = minabs1;
            }
        } else {
            // 0-bit is impossible (would make the value zero),
            // so assume the bit is 1 without reading it
            have = minabs1;
        }
    }
    let have = I::from(have).unwrap();
    Ok(if sign { have } else { (!have) + I::one() })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bitwise_negation() {
        let n = -6;
        assert_eq!(6, !n + 1);
    }
}
