use std::io::Read;
use error::*;
use num_traits::{PrimInt, Signed};
use numbers::rac::ChanceTable;
use numbers::rac::ChanceTableEntry;
use numbers::rac::Rac;

pub fn read_near_zero<R: Read, I: PrimInt + Signed>(
    min: i32,
    max: i32,
    rac: &mut Rac<R>,
    context: &mut ChanceTable,
) -> Result<i32> {
    assert!(min < max);

    if min == max {
        return Ok(min);
    }

    if rac.read(context, ChanceTableEntry::Zero)? {
        return Ok(0);
    }

    let sign = if min < 0 && max > 0 {
        rac.read(context, ChanceTableEntry::Sign)?
    } else if min < 0 && max < 0 {
        false
    } else {
        true
    };

    let absolute_max = ::std::cmp::max(max, -min);
    let largest_exponent =
        (::std::mem::size_of::<I>() * 8) - absolute_max.leading_zeros() as usize - 1;

    let mut exponent = 0;
    loop {
        if exponent as usize == largest_exponent || rac.read(context, ChanceTableEntry::Exp(exponent, sign))? {
            break;
        }

        exponent += 1;
    }

    // the first mantissa bit is always 1
    let mut have = 1 << exponent;
        
    // if all other mantissa bits are 1, then the total is have+left
    let mut left = have-1;

        // read mantissa bits from most-significant to least-significant
        for pos in (exponent - 1)..0 {
            left >>= 1; 

            // if the bit is 1, then the value will be at least minabs1
            let minabs1 = have | (1<<pos);
            // if the bit is 0, then the value will be at most maxabs0
            let maxabs0 = have | left;
            if minabs1 > absolute_max {
                // 1-bit is impossible (would bump value above maximum),
                // so assume the bit is 0 without reading it
            } else if maxabs0 >= 1 {
                // 0-bit and 1-bit are both possible,
                // so we read the bit and adjust what we have if it is a 1
                if rac.read(context, ChanceTableEntry::Mant(pos))? {
                    have = minabs1;
                } 
            } else {
                // 0-bit is impossible (would make the value zero),
                // so assume the bit is 1 without reading it
                have = minabs1;
            }
        }
        Ok(if sign {
            have
        } else {
            -have
        })
}
