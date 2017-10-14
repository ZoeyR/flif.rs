use std::marker::PhantomData;
use std::io::Read;
use error::*;
use private::Sealed;
use super::FlifReadExt;

pub trait Config: Sealed {
    const MAX_RANGE_BITS: u32;
    const MIN_RANGE_BITS: u32;
    const MIN_RANGE: u32 = 1 << Self::MIN_RANGE_BITS;
    const BASE_RANGE: u32 = 1 << Self::MAX_RANGE_BITS;

    fn apply_chance(chance: u32, range: u32) -> u32;
}

pub struct Config24;

impl Config for Config24 {
    const MAX_RANGE_BITS: u32 = 24;
    const MIN_RANGE_BITS: u32 = 16;

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
        let lower_12bits = (((range & 0xFFF) * chance) + 2048) >> 12;
        let upper_bits = (range >> 12) * chance;
        upper_bits + lower_12bits
    }
}

impl Sealed for Config24 {}

pub struct Input<C, R> {
    reader: R,
    config: PhantomData<C>,
    range: u32,
    low: u32,
}

impl<C, R> Input<C, R>
where
    C: Config,
    R: Read,
{
    pub fn new(mut reader: R) -> Result<Input<C, R>> {
        //TODO figure out how to make this code cleaner

        // calculate the number of iterations needed to calculate low. The number of iterations
        // should be C::max_range_bits() / 8 rounded up
        let needed_iterations =
            (C::MAX_RANGE_BITS / 8) + if C::MAX_RANGE_BITS % 8 > 0 { 1 } else { 0 };

        let low = (0..needed_iterations).fold(Ok(0), |acc: Result<u32>, _| {
            let or_val = reader.read_u8()?;
            acc.map(|acc| (acc << 8) | or_val as u32)
        })?;

        Ok(Input {
            reader: reader,
            config: PhantomData,
            range: C::BASE_RANGE,
            low,
        })
    }

    fn input(&mut self) -> Result<()> {
        for _ in 0..2 {
            if self.range <= C::MIN_RANGE {
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

    pub fn read_bit(&mut self) -> Result<bool> {
        // creates a 50% chance
        let chance = self.range >> 1;
        self.get(chance)
    }
}
