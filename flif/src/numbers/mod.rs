use std::io::prelude::*;
use error::*;
use num_traits::{PrimInt, Unsigned};

pub mod chances;
pub mod near_zero;
pub mod rac;
pub mod symbol;
pub mod varint;

pub trait FlifReadExt {
    fn read_u8(&mut self) -> Result<u8>;
    fn read_varint<T: PrimInt + Unsigned>(&mut self) -> Result<T>;
}

impl<R: Read> FlifReadExt for R {
    fn read_u8(&mut self) -> Result<u8> {
        let mut byte_buf = [0; 1];
        self.read_exact(&mut byte_buf)?;
        Ok(byte_buf[0])
    }

    fn read_varint<T: PrimInt + Unsigned>(&mut self) -> Result<T> {
        varint::read_varint(self)
    }
}

pub fn median3<T: PrimInt>(first: T, second: T, third: T) -> T {
    let mut slice = [first, second, third];
    slice.sort();
    slice[1]
}
