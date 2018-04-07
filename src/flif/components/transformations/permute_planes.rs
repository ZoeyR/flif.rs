use super::Transform;
use ColorValue;
use components::transformations::ColorRange;
use error::*;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::near_zero::NearZeroCoder;
use numbers::rac::RacRead;

pub struct PermutePlanes<T: Transform> {
    channel_map: Vec<usize>,
    previous_transform: T,
    range_function: fn(&T, usize, &[usize]) -> ColorRange,
    crange_function: fn(&T, usize, &[ColorValue], &[usize]) -> ColorRange,
}

impl<T: Transform> PermutePlanes<T> {
    pub fn new<R: RacRead>(
        rac: &mut R,
        previous_transform: T,
        channels: usize,
        update_table: &UpdateTable,
    ) -> Result<PermutePlanes<T>> {
        let mut context = ChanceTable::new(update_table);

        let subtract = rac.read_near_zero(0, 1, &mut context)? == 1;

        let range_function = if subtract {
            subtract_range
        } else {
            normal_range
        };

        let crange_function = if subtract {
            subtract_crange
        } else {
            normal_crange
        };

        let channel_map: Vec<_> = (0..channels)
            .map(|_| rac.read_near_zero(0, channels - 1, &mut context))
            .collect::<Result<_>>()?;

        Ok(PermutePlanes {
            channel_map,
            previous_transform,
            range_function,
            crange_function,
        })
    }
}

impl<T: Transform> Transform for PermutePlanes<T> {
    fn undo(&self, _pixel: &mut [ColorValue]) {
        unimplemented!()
    }

    fn range(&self, channel: usize) -> ColorRange {
        (self.range_function)(&self.previous_transform, channel, &self.channel_map)
    }

    fn crange(&self, channel: usize, values: &[ColorValue]) -> ColorRange {
        (self.crange_function)(&self.previous_transform, channel, values, &self.channel_map)
    }
}

fn subtract_range<T: Transform>(
    transform: &T,
    channel: usize,
    permutation: &[usize],
) -> ColorRange {
    if channel == 0 || channel == 3 {
        transform.range(permutation[channel])
    } else {
        let channel_range = transform.range(permutation[channel]);
        let zero_range = transform.range(permutation[0]);
        ColorRange {
            min: channel_range.min - zero_range.max,
            max: channel_range.max - zero_range.min,
        }
    }
}

fn subtract_crange<T: Transform>(
    transform: &T,
    channel: usize,
    values: &[ColorValue],
    permutation: &[usize],
) -> ColorRange {
    let mut range = subtract_range(transform, channel, permutation);
    range.min -= values[0];
    range.max -= values[0];
    range
}

fn normal_range<T: Transform>(transform: &T, channel: usize, permutation: &[usize]) -> ColorRange {
    transform.range(permutation[channel])
}

fn normal_crange<T: Transform>(
    transform: &T,
    channel: usize,
    _values: &[ColorValue],
    permutation: &[usize],
) -> ColorRange {
    normal_range(transform, channel, permutation)
}
