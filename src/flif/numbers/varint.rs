use std::io::Read;
use error::{Error, Result};
use num_traits::{PrimInt, Unsigned};
use super::FlifReadExt;

// T::from(_).unwrap() is panic-safe in this function because there exists no type that is both
// PrimInt and Unsigned that cannot store a u8
pub fn read_varint<R: Read, T: PrimInt + Unsigned>(mut reader: R) -> Result<T> {
    let bitshift_multiplier = &T::from(128).unwrap();
    let mut acc: T = T::zero();

    loop {
        let byte = reader.read_u8()?;

        // This is the last segment of the varint
        if byte < 0b1000_0000 {
            break Ok(acc + T::from(byte).unwrap());
        }

        let byte = byte & 0b0111_1111;
        if let Some(val) = acc.checked_add(&T::from(byte).unwrap())
            .and_then(|val| val.checked_mul(bitshift_multiplier))
        {
            acc = val;
        } else {
            break Err(Error::InvalidVarint);
        }
    }
}
