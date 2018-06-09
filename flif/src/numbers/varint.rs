use std::io::Read;

use super::FlifReadExt;
use error::{Error, Result};
use num_traits::{PrimInt, Unsigned};

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
        if let Some(val) = acc
            .checked_add(&T::from(byte).unwrap())
            .and_then(|val| val.checked_mul(bitshift_multiplier))
        {
            acc = val;
        } else {
            break Err(Error::InvalidVarint);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_varint_read() {
        use numbers::FlifReadExt;

        let buf = [0x82, 0x5F, 0x82, 0x2F];

        let first: u32 = buf.as_ref().read_varint().unwrap();
        let second: u32 = buf[2..].as_ref().read_varint().unwrap();
        assert_eq!(first, 351);
        assert_eq!(second, 303);
    }

    #[test]
    fn test_varint_max_read() {
        use numbers::FlifReadExt;

        let buf = [0x8F, 0xFF, 0xFF, 0xFF, 0x7F];
        let num: u32 = buf.as_ref().read_varint().unwrap();
        assert_eq!(num, u32::max_value());
    }

    #[test]
    fn test_varint_min_read() {
        use numbers::FlifReadExt;

        let buf = [0x00];
        let num: u32 = buf.as_ref().read_varint().unwrap();
        assert_eq!(num, u32::min_value());
    }

    #[test]
    fn test_varint_overflow_read() {
        use error::*;
        use numbers::FlifReadExt;

        let buf = [0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        let result: Result<u32> = buf.as_ref().read_varint();

        assert_eq!(
            ::std::mem::discriminant(&result.unwrap_err()),
            ::std::mem::discriminant(&Error::InvalidVarint)
        )
    }
}
