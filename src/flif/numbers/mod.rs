use std::io::prelude::*;

use error::*;

pub mod rac;
pub mod varint;
pub mod symbol;

pub trait FlifReadExt {
    fn read_u8(&mut self) -> Result<u8>;
    fn read_varint(&mut self) -> Result<u32>;
}

impl<R: Read> FlifReadExt for R {
    fn read_u8(&mut self) -> Result<u8> {
        let mut byte_buf = [0; 1];
        self.read_exact(&mut byte_buf)?;
        Ok(byte_buf[0])
    }

    fn read_varint(&mut self) -> Result<u32> {
        varint::read_varint(self)
    }
}
