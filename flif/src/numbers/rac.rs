use std::io;
use std::io::Read;
#[cfg(test)]
use std::io::Write;

use super::FlifReadExt;
use crate::error::*;
use crate::numbers::chances::{ChanceTable, ChanceTableEntry};

pub trait RacRead {
    fn read_bit(&mut self) -> Result<bool>;
    fn read_chance(&mut self, chance: u32) -> Result<bool>;
    fn read(&mut self, context: &mut ChanceTable, entry: ChanceTableEntry) -> Result<bool>;
}

#[derive(Debug)]
pub struct Rac<RW> {
    reader: RW,
    range: u32,
    low: u32,
}

impl<RW> Rac<RW> {
    const MAX_RANGE_BITS: u32 = 24;
    const MIN_RANGE_BITS: u32 = 16;
    const MIN_RANGE: u32 = 1 << Self::MIN_RANGE_BITS;
    const MAX_RANGE: u32 = 1 << Self::MAX_RANGE_BITS;

    /// chances are a number between 0 and 4096, this function expands that into range, e.g.
    /// let x = chance / 4096
    /// chance_12bit_chance returns an integer ciel(y) such that y/range = x
    /// in otherwords, chance_12bit_chance(chance, range) = ciel((chance / 4096) * range)
    fn apply_chance(chance: u32, range: u32) -> u32 {
        // this should never happen via a malicious file, the only callers of this get their chances from the chance table
        assert_eq!(chance >> 12, 0);

        // there is the possibility of integer overflow so we break up the calculation to prevent
        // overflow by applying the following tranformations to the formula
        // range = (range >> 12) + (range & 0xFFF)
        // range * chance = ((range >> 12) * chance) + ((range & 0xFFF) * chance)
        // range * chance / 4096 = range / 4096 * chance
        let lower_12bits = (((range & 0xFFF) * chance) + 2048) / 4096;
        let upper_bits = (range / 4096) * chance;
        upper_bits + lower_12bits
    }
}

impl<R: Read> RacRead for Rac<R> {
    fn read_bit(&mut self) -> Result<bool> {
        // creates a 50% chance
        let chance = self.range >> 1;
        self.get(chance)
    }

    fn read_chance(&mut self, chance: u32) -> Result<bool> {
        let chance = Self::apply_chance(chance, self.range);
        self.get(chance)
    }

    #[inline(always)]
    fn read(&mut self, context: &mut ChanceTable, entry: ChanceTableEntry) -> Result<bool> {
        let chance = context.get_chance(entry);
        let transformed_chance = Self::apply_chance(u32::from(chance), self.range);
        let bit = self.get(transformed_chance)?;
        context.update_entry(bit, entry);

        Ok(bit)
    }
}

impl<R: Read> Rac<R> {
    pub fn from_reader(mut reader: R) -> Result<Rac<R>> {
        // calculate the number of iterations needed to calculate low. The number of iterations
        // should be Self::MAX_RANGE_BITS / 8 rounded up
        let needed_iterations =
            (Self::MAX_RANGE_BITS / 8) + if Self::MAX_RANGE_BITS % 8 > 0 { 1 } else { 0 };

        let low = (0..needed_iterations).fold(Ok(0), |acc: Result<u32>, _| {
            let or_val = match reader.read_u8() {
                Ok(val) => val,
                Err(Error::Io(ref io)) if io.kind() == io::ErrorKind::UnexpectedEof => 0xFF,
                err => err?,
            };
            acc.map(|acc| (acc << 8) | u32::from(or_val))
        })?;

        Ok(Rac {
            reader,
            range: Self::MAX_RANGE,
            low,
        })
    }

    fn input(&mut self) -> Result<()> {
        for _ in 0..2 {
            if self.range <= Self::MIN_RANGE {
                self.low <<= 8;
                self.range <<= 8;

                self.low |= u32::from(match self.reader.read_u8() {
                    Ok(val) => val,
                    Err(Error::Io(ref io)) if io.kind() == io::ErrorKind::UnexpectedEof => 0xFF,
                    err => err?,
                });
            }
        }
        Ok(())
    }

    fn get(&mut self, chance: u32) -> Result<bool> {
        // this should never happen via a malicious file,
        // the chance here should be produced by our projection
        // function and represents a programming error.
        assert!(chance < self.range);

        if self.low >= self.range - chance {
            self.low -= self.range - chance;
            self.range = chance;
            self.input()?;
            Ok(true)
        } else {
            self.range -= chance;
            self.input()?;
            Ok(false)
        }
    }
}

#[cfg(test)]
impl<W: Write> Rac<W> {
    pub fn from_writer(writer: W) -> Rac<W> {
        Rac {
            low: 0,
            range: Self::MAX_RANGE,
            reader: writer,
        }
    }

    fn output(&mut self) -> Result<()> {
        if self.range <= Self::MIN_RANGE {
            // write out the top 8 bits of low
            let byte = (self.low >> Self::MIN_RANGE_BITS) as u8;
            self.reader.write_all(&[byte])?;
            self.low <<= 8;
            self.range <<= 8;
        }

        Ok(())
    }

    fn set(&mut self, chance: u32, bit: bool) -> Result<()> {
        if bit {
            self.low += self.range - chance;
            self.range = chance;
            self.output()?;
        } else {
            self.range -= chance;
            self.output()?;
        }

        Ok(())
    }

    pub fn write_bit(&mut self, bit: bool) -> Result<()> {
        let chance = self.range >> 1;
        self.set(chance, bit)
    }

    pub fn write_chance(&mut self, chance: u32, bit: bool) -> Result<()> {
        let chance = Self::apply_chance(chance, self.range);
        self.set(chance, bit)
    }

    pub fn flush(&mut self) -> Result<()> {
        // flush is only ever required if there is data in the top 8 bits (out of 24) of low.
        if self.low >> Self::MIN_RANGE_BITS > 0 {
            let byte = (self.low >> Self::MIN_RANGE_BITS) as u8;
            self.reader.write_all(&[byte])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    const BITS: [(u32, bool); 8] = [
        (3333, true),
        (567, false),
        (123, false),
        (3990, true),
        (1, false),
        (4000, true),
        (2780, true),
        (4095, true),
    ];

    #[test]
    fn test_rac_bidirectional_chance() {
        use crate::numbers::rac::{Rac, RacRead};

        let mut buf: Vec<u8> = vec![];
        {
            let mut writer_rac = Rac::from_writer(&mut buf);
            for &(chance, bit) in BITS.iter() {
                writer_rac.write_chance(chance, bit).unwrap();
            }
            writer_rac.flush().unwrap();
        }

        let read_buf: &[u8] = buf.as_ref();
        let mut reader_rac = Rac::from_reader(read_buf).unwrap();
        for &(chance, bit) in BITS.iter() {
            assert_eq!(bit, reader_rac.read_chance(chance).unwrap());
        }
    }

    #[test]
    fn test_rac_bidirectional_bits() {
        use crate::numbers::rac::{Rac, RacRead};

        let mut buf: Vec<u8> = vec![];
        {
            let mut writer_rac = Rac::from_writer(&mut buf);
            for &(_, bit) in BITS.iter() {
                writer_rac.write_bit(bit).unwrap();
            }
            writer_rac.flush().unwrap();
        }

        let read_buf: &[u8] = buf.as_ref();
        let mut reader_rac = Rac::from_reader(read_buf).unwrap();
        for &(_, bit) in BITS.iter() {
            assert_eq!(bit, reader_rac.read_bit().unwrap());
        }
    }
}
