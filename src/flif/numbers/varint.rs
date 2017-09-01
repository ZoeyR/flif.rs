use std::io::Read;

use error::*;
use super::FlifReadExt;

pub fn read_varint<R: Read>(mut reader: R) -> Result<u32> {
    let mut acc: u32 = 0;
    // The specification is vague on the exact properties of varints. For the purposes of this
    // implementation we will assume that a varint can be stored in an unsigned 32bit number.
    // We could also assume that the varint was maximally compact, if we did so we could change
    // the following loop to only read 5 bytes, however, the reference implementation loops for
    // a maximum of 10 bytes so we will do the same for compatibility.
    for _ in 0..10 {
        let byte = reader.read_u8()?;

        // This is the last segment of the varint
        if byte < 0b1000_0000 {
            return Ok(acc + byte as u32);
        }

        // We are still in the middle of the number.
        // Mask out the top bit so it doesn't get included in the calculation.
        let byte = byte & 0b0111_1111;
        acc += byte as u32;
        acc <<= 7;
    }

    // We got here without actually finding the end of the number, we need to indicate an
    // error.
    Err(Error::InvalidHeader {
        desc: "reader did not contain a varint",
    })
}
