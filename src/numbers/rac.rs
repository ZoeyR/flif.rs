use std::marker::PhantomData;
use std::io::Read;

use error::*;
use super::FlifReadExt;
use private::Sealed;

pub trait Config: Sealed {
    #[inline]
    fn max_range_bits() -> u32;
    #[inline]
    fn min_range_bits() -> u32;
    #[inline]
    fn min_range() -> u32 {
        1 << Self::min_range_bits()
    }
    #[inline]
    fn base_range() -> u32 {
        1 << Self::max_range_bits()
    }
    fn chance_12bit_chance(b12: u32, range: u32) -> Result<u32>;
}

pub struct Config24;

impl Config for Config24 {
    fn max_range_bits() -> u32 {
        24
    }

    fn min_range_bits() -> u32 {
        16
    }

    fn chance_12bit_chance(b12: u32, range: u32) -> Result<u32> {
        assert_eq!(b12 >> 12, 0);

        Ok(
            (((range & 0xFFF) * b12 + 0x800) >> 12) + ((range >> 12) * b12),
        )
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
            (C::max_range_bits() / 8) + if C::max_range_bits() % 8 > 0 { 1 } else { 0 };

        let low = (0..needed_iterations).fold(Ok(0), |acc: Result<u32>, _| {
            let or_val = reader.read_u8()?;
            acc.map(|acc| (acc << 8) | or_val as u32)
        })?;

        Ok(Input {
            reader: reader,
            config: PhantomData,
            range: C::base_range(),
            low,
        })
    }

    fn input(&mut self) -> Result<()> {
        for _ in 0..2 {
            if self.range <= C::min_range() {
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
        let chance = self.range >> 1;
        self.get(chance)
    }
}
