use std::io::Write;
use std::collections::HashMap;
use std::io::Read;
use std::io;
use error::*;
use super::FlifReadExt;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum ChanceTableEntry {
    Zero,
    Sign,
    Exp(u8, bool),
    Mant(u8),
}

pub struct ChanceTable {
    table: HashMap<ChanceTableEntry, u16>,
    update_table: Vec<u16>,
}

impl ChanceTable {
    pub fn new(alpha_divisor: u8, cutoff: u8) -> ChanceTable {
        let update_table = Self::build_update_table(alpha_divisor, cutoff);
        let mut table = HashMap::new();
        table.insert(ChanceTableEntry::Zero, 1000);
        table.insert(ChanceTableEntry::Sign, 2048);
        Self::insert_exp(&mut table, false);
        Self::insert_exp(&mut table, true);
        table.insert(ChanceTableEntry::Mant(0), 1900);
        table.insert(ChanceTableEntry::Mant(1), 1850);
        table.insert(ChanceTableEntry::Mant(2), 1800);
        table.insert(ChanceTableEntry::Mant(3), 1750);
        table.insert(ChanceTableEntry::Mant(4), 1650);
        table.insert(ChanceTableEntry::Mant(5), 1600);
        table.insert(ChanceTableEntry::Mant(6), 1600);
        table.insert(ChanceTableEntry::Mant(7), 2048);

        ChanceTable {
            table,
            update_table,
        }
    }

    pub fn get_chance(&mut self, entry: ChanceTableEntry) -> u16 {
        *self.table.entry(entry).or_insert(2048)
    }

    pub fn update_entry(&mut self, bit: bool, entry: ChanceTableEntry) {
        let new_chance = {
            let old_chance = self.table.entry(entry).or_insert(2048);

            if bit {
                self.update_table[*old_chance as usize]
            } else {
                4096 - self.update_table[(4096 - *old_chance) as usize]
            }
        };

        self.table.insert(entry, new_chance);
    }

    fn insert_exp(table: &mut HashMap<ChanceTableEntry, u16>, sign: bool) {
        table.insert(ChanceTableEntry::Exp(0, sign), 1000);
        table.insert(ChanceTableEntry::Exp(1, sign), 1200);
        table.insert(ChanceTableEntry::Exp(2, sign), 1500);
        table.insert(ChanceTableEntry::Exp(3, sign), 1750);
        table.insert(ChanceTableEntry::Exp(4, sign), 2000);
        table.insert(ChanceTableEntry::Exp(5, sign), 2300);
        table.insert(ChanceTableEntry::Exp(6, sign), 2800);
        table.insert(ChanceTableEntry::Exp(7, sign), 2400);
        table.insert(ChanceTableEntry::Exp(8, sign), 2300);
        table.insert(ChanceTableEntry::Exp(9, sign), 2048);
    }

    fn build_update_table(alpha_divisor: u8, cutoff: u8) -> Vec<u16> {
        let mut update_table = vec![0; 4096];

        let max_chance: u16 = 4096 - cutoff as u16;
        let mut old_chance: u16 = 0;

        let mut chance_accumulator: u64 = u32::max_value() as u64;
        for _ in 0..2048 {
            let mut new_chance: u16 = (chance_accumulator >> 24) as u16
                + if chance_accumulator & 0xFFFFFF > 0 {
                    1
                } else {
                    0
                };

            if new_chance < old_chance {
                new_chance = old_chance + 1;
            }

            if (old_chance != 0) && ((old_chance as usize) < update_table.len())
                && new_chance <= max_chance as u16
            {
                update_table[old_chance as usize] = new_chance;
            }

            chance_accumulator +=
                Self::update_chance_accumulator(chance_accumulator, alpha_divisor);
            old_chance = new_chance;
        }

        // fill in the rest of the table
        for old_chance in cutoff as u16..(max_chance + 1) {
            const MAX: u64 = 1 + ::std::u32::MAX as u64;
            if update_table[old_chance as usize] != 0 {
                continue;
            }

            let mut new_chance = (old_chance as u64 * MAX + 2048) / 4096;
            new_chance += Self::update_chance_accumulator(new_chance, alpha_divisor);
            new_chance = (4096 * new_chance + (MAX / 2)) >> 32;

            if new_chance <= old_chance as u64 {
                new_chance = old_chance as u64 + 1;
            }

            if new_chance > max_chance as u64 {
                new_chance = max_chance as u64;
            }
            update_table[old_chance as usize] = new_chance as u16;
        }

        update_table
    }

    #[inline(always)]
    fn update_chance_accumulator(old: u64, alpha_divisor: u8) -> u64 {
        const MAX: u64 = ::std::u32::MAX as u64;
        let v = (MAX - old as u64 + 1) * (MAX / alpha_divisor as u64);
        if v & 0xFFFFFFFF > 0 {
            ((v + 1) >> 32) as u64
        } else {
            (v >> 32) as u64
        }
    }
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
        assert_eq!(chance >> 12, 0);
        //assert_eq!(range >> 24, 0);

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

                self.low |= match self.reader.read_u8() {
                    Ok(val) => val,
                    Err(Error::Io(ref io)) if io.kind() == io::ErrorKind::UnexpectedEof => 0xFF,
                    err => err?,
                } as u32;
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

    pub fn read_chance(&mut self, chance: u32) -> Result<bool> {
        let chance = Self::apply_chance(chance, self.range);
        self.get(chance)
    }

    pub fn read(&mut self, context: &mut ChanceTable, entry: ChanceTableEntry) -> Result<bool> {
        let chance = context.get_chance(entry);
        let bit = self.get(chance as u32)?;
        context.update_entry(bit, entry);

        Ok(bit)
    }
}

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
            self.reader.write(&[byte])?;
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
            self.reader.write(&[byte])?;
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
        use numbers::rac::Rac;

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
        use numbers::rac::Rac;

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
