use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ChanceTable<'a> {
    table: HashMap<ChanceTableEntry, u16>,
    updates: &'a UpdateTable,
}

impl<'a> ChanceTable<'a> {
    pub fn new(updates: &UpdateTable) -> ChanceTable {
        //let update_table = Self::build_update_table(alpha_divisor, cutoff);
        let mut table = HashMap::new();
        table.insert(ChanceTableEntry::Zero, 1000);
        table.insert(ChanceTableEntry::Sign, 2048);
        Self::insert_exp(&mut table, false);
        Self::insert_exp(&mut table, true);
        Self::insert_mant(&mut table);

        ChanceTable { table, updates }
    }

    pub fn get_chance(&mut self, entry: ChanceTableEntry) -> u16 {
        *self.table.entry(entry).or_insert(2048)
    }

    pub fn update_entry(&mut self, bit: bool, entry: ChanceTableEntry) {
        let old_chance = *self.table.entry(entry).or_insert(2048);
        let new_chance = self.updates.next_chance(bit, old_chance);

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

    fn insert_mant(table: &mut HashMap<ChanceTableEntry, u16>) {
        table.insert(ChanceTableEntry::Mant(0), 1900);
        table.insert(ChanceTableEntry::Mant(1), 1850);
        table.insert(ChanceTableEntry::Mant(2), 1800);
        table.insert(ChanceTableEntry::Mant(3), 1750);
        table.insert(ChanceTableEntry::Mant(4), 1650);
        table.insert(ChanceTableEntry::Mant(5), 1600);
        table.insert(ChanceTableEntry::Mant(6), 1600);
        table.insert(ChanceTableEntry::Mant(7), 2048);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ChanceTableEntry {
    Zero,
    Sign,
    Exp(u8, bool),
    Mant(u8),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UpdateTable {
    updates: Vec<u16>,
}

impl UpdateTable {
    pub fn new(alpha_divisor: u8, cutoff: u8) -> UpdateTable {
        let mut updates = vec![0; 4096];

        let max_chance: u16 = 4096 - cutoff as u16;
        let mut old_chance: u16 = 0;

        let mut chance_accumulator: u64 = 1 << 31;
        for _ in 0..2048 {
            let mut new_chance: u16 = (chance_accumulator >> 20) as u16
                + if chance_accumulator & 0x80000 > 0 {
                    1
                } else {
                    0
                };

            if new_chance <= old_chance {
                new_chance = old_chance + 1;
            }

            if (old_chance != 0) && ((old_chance as usize) < updates.len())
                && new_chance <= max_chance as u16
            {
                updates[old_chance as usize] = new_chance;
            }

            chance_accumulator +=
                Self::update_chance_accumulator(chance_accumulator, alpha_divisor);
            old_chance = new_chance;
        }

        //fill in the rest of the table
        for old_chance in cutoff as u16..(max_chance + 1) {
            const MAX: u64 = 1 + ::std::u32::MAX as u64;
            if updates[old_chance as usize] != 0 {
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
            updates[old_chance as usize] = new_chance as u16;
        }

        UpdateTable { updates }
    }

    pub fn next_chance(&self, bit: bool, chance: u16) -> u16 {
        if bit {
            self.updates[chance as usize]
        } else {
            4096 - self.updates[(4096 - chance) as usize]
        }
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
