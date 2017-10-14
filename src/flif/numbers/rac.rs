use std::io::Read;
use error::*;
use super::FlifReadExt;

pub struct Rac<R> {
    reader: R,
    range: u32,
    low: u32,
}

impl<R: Read> Rac<R> {
    const MAX_RANGE_BITS: u32 = 24;
    const MIN_RANGE_BITS: u32 = 16;
    const MIN_RANGE: u32 = 1 << Self::MIN_RANGE_BITS;
    const MAX_RANGE: u32 = 1 << Self::MAX_RANGE_BITS;

    pub fn new(mut reader: R) -> Result<Rac<R>> {
        // calculate the number of iterations needed to calculate low. The number of iterations
        // should be Self::MAX_RANGE_BITS / 8 rounded up
        let needed_iterations =
            (Self::MAX_RANGE_BITS / 8) + if Self::MAX_RANGE_BITS % 8 > 0 { 1 } else { 0 };

        let low = (0..needed_iterations).fold(Ok(0), |acc: Result<u32>, _| {
            let or_val = reader.read_u8()?;
            acc.map(|acc| (acc << 8) | or_val as u32)
        })?;

        Ok(Rac {
            reader: reader,
            range: Self::MAX_RANGE,
            low,
        })
    }

    fn input(&mut self) -> Result<()> {
        for _ in 0..2 {
            if self.range <= Self::MIN_RANGE {
                self.low <<= 8;
                self.range <<= 8;
                self.low |= self.reader.read_u8()? as u32;
            }
        }
        Ok(())
    }

    fn get(&mut self, chance: u32) -> Result<bool> {
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

    /// chances are a number between 0 and 4096, this function expands that into range, e.g.
    /// let x = chance / 4096
    /// chance_12bit_chance returns an integer ciel(y) such that y/range = x
    /// in otherwords, chance_12bit_chance(chance, range) = ciel((chance / 4096) * range)
    fn apply_chance(chance: u32, range: u32) -> u32 {
        assert_eq!(chance >> 12, 0);
        assert_eq!(range >> 24, 0);

        // there is the possibility of integer overflow so we break up the calculation to prevent
        // overflow by applying the following tranformations to the formula
        // range = (range >> 12) + (range & 0xFFF)
        // range * chance = ((range >> 12) * chance) + ((range & 0xFFF) * chance)
        // range * chance / 4096 = range / 4096 * chance
        let lower_12bits = (((range & 0xFFF) * chance) + 2048) / 4096;
        let upper_bits = (range / 4096) * chance;
        upper_bits + lower_12bits
    }

    pub fn read_bit(&mut self) -> Result<bool> {
        // creates a 50% chance
        let chance = self.range >> 1;
        self.get(chance)
    }

    pub fn read_chance(&mut self, chance: u32) -> Result<bool> {
        let chance = Self::apply_chance(chance, self.range);
        self.get(chance)
    }
}
