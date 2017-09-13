use std::io::Read;

use num_traits::{PrimInt, Unsigned};

use error::*;
use super::FlifReadExt;

pub fn read_varint<R: Read, T: PrimInt + Unsigned>(mut reader: R) -> Result<T> {
    let mut acc: T = T::zero();
    // The specification is vague on the exact properties of varints. For the purposes of this
    // implementation we will assume that a varint can be stored in an unsigned 32bit number.
    // We could also assume that the varint was maximally compact, if we did so we could change
    // the following loop to only read 5 bytes, however, the reference implementation loops for
    // a maximum of 10 bytes so we will do the same for compatibility.
    loop {
        let byte = reader.read_u8()?;

        // This is the last segment of the varint
        if byte < 0b1000_0000 {
            break Ok(acc + T::from(byte).unwrap());
        }

        let byte = byte & 0b0111_1111;
        if let Some(val) = acc.checked_add(&T::from(byte).unwrap())
            .and_then(|val| val.checked_mul(&T::from(128).unwrap()))
        {
            acc = val;
        } else {
            break Err(Error::InvalidHeader {
                desc: "reader did not contain a varint, or varint was too large for u32",
            });
        }
    }
}
