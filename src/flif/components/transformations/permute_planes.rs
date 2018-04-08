use super::Transform;
use components::transformations::ColorRange;
use error::*;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::near_zero::NearZeroCoder;
use numbers::rac::RacRead;
use colors::{Channel, ChannelSet, ColorSpace, Pixel};

pub struct PermutePlanes<T: Transform> {
    channel_set: ChannelSet<Channel>,
    previous_transform: T,
    range_function: fn(&T, Channel, &ChannelSet<Channel>) -> ColorRange,
    crange_function: fn(&T, Channel, &Pixel, &ChannelSet<Channel>) -> ColorRange,
}

impl<T: Transform> PermutePlanes<T> {
    pub fn new<R: RacRead>(
        rac: &mut R,
        previous_transform: T,
        channels: ColorSpace,
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

        let mut channel_set: ChannelSet<Channel> = Default::default();
        for channel in channels {
            let channel_number = rac.read_near_zero(0, channels as usize - 1, &mut context)?;
            let mapped_channel = Channel::from_number(channel_number)?;
            channel_set[channel] = mapped_channel;
        }

        Ok(PermutePlanes {
            channel_set,
            previous_transform,
            range_function,
            crange_function,
        })
    }
}

impl<T: Transform> Transform for PermutePlanes<T> {
    fn undo(&self, _pixel: &mut Pixel) {
        unimplemented!()
    }

    fn range(&self, channel: Channel) -> ColorRange {
        (self.range_function)(&self.previous_transform, channel, &self.channel_set)
    }

    fn crange(&self, channel: Channel, values: &Pixel) -> ColorRange {
        (self.crange_function)(&self.previous_transform, channel, values, &self.channel_set)
    }
}

fn subtract_range<T: Transform>(
    transform: &T,
    channel: Channel,
    permutation: &ChannelSet<Channel>,
) -> ColorRange {
    if channel == Channel::Red || channel == Channel::Alpha {
        transform.range(permutation[channel])
    } else {
        let channel_range = transform.range(permutation[channel]);
        let zero_range = transform.range(permutation[Channel::Red]);
        ColorRange {
            min: channel_range.min - zero_range.max,
            max: channel_range.max - zero_range.min,
        }
    }
}

fn subtract_crange<T: Transform>(
    transform: &T,
    channel: Channel,
    values: &Pixel,
    permutation: &ChannelSet<Channel>,
) -> ColorRange {
    let mut range = subtract_range(transform, channel, permutation);
    range.min -= values[Channel::Red];
    range.max -= values[Channel::Red];
    range
}

fn normal_range<T: Transform>(
    transform: &T,
    channel: Channel,
    permutation: &ChannelSet<Channel>,
) -> ColorRange {
    transform.range(permutation[channel])
}

fn normal_crange<T: Transform>(
    transform: &T,
    channel: Channel,
    _values: &Pixel,
    permutation: &ChannelSet<Channel>,
) -> ColorRange {
    normal_range(transform, channel, permutation)
}
