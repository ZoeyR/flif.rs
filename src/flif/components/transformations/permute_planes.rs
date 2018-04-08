use super::Transform;
use components::transformations::ColorRange;
use error::*;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::near_zero::NearZeroCoder;
use numbers::rac::RacRead;
use colors::{Channel, ChannelSet, ColorSpace, Pixel};

pub struct PermutePlanes {
    channels: ColorSpace,
    channel_set: ChannelSet<Channel>,
    ranges: ChannelSet<ColorRange>,
    crange_function: fn(&PermutePlanes, Channel, &Pixel) -> ColorRange,
}

impl PermutePlanes {
    pub fn new<T: Transform, R: RacRead>(
        rac: &mut R,
        previous_transform: T,
        channels: ColorSpace,
        update_table: &UpdateTable,
    ) -> Result<PermutePlanes> {
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

        let mut ranges: ChannelSet<ColorRange> = Default::default();
        for channel in channels {
            ranges[channel] = (range_function)(&previous_transform, channel, &channel_set);
        }

        Ok(PermutePlanes {
            channels,
            channel_set,
            ranges,
            crange_function,
        })
    }
}

impl Transform for PermutePlanes {
    fn undo(&self, pixel: &mut Pixel) {
        let mut new_pixel = Pixel::default();
        for channel in self.channels {
            let permuted_channel = self.channel_set[channel];
            new_pixel[permuted_channel] = pixel[channel];
        }

        *pixel = new_pixel;
    }

    fn range(&self, channel: Channel) -> ColorRange {
        self.ranges[channel]
    }

    fn crange(&self, channel: Channel, values: &Pixel) -> ColorRange {
        (self.crange_function)(self, channel, values)
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

fn subtract_crange(transform: &PermutePlanes, channel: Channel, values: &Pixel) -> ColorRange {
    let mut range = transform.ranges[channel];
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

fn normal_crange(transform: &PermutePlanes, channel: Channel, _values: &Pixel) -> ColorRange {
    transform.ranges[channel]
}
