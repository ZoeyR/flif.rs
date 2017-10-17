use super::Transformation;

pub struct ChannelCompact {
    min: u16,
    max: u16,
    cmin: u16,
    cmax: u16,
}
impl ChannelCompact {
    pub fn new<T: Transformation>(transformation: &T, num_channels: u8) -> ChannelCompact {
        ChannelCompact {
            min: 0,
            max: 0,
            cmin: 0,
            cmax: 0,
        }
    }
}

impl Transformation for ChannelCompact {
    fn snap(&self, channel: u8, values: u16, pixel: u16) -> u16 {
        unimplemented!()
    }

    fn min(&self, channel: u8) -> u16 {
        unimplemented!()
    }

    fn max(&self, channel: u8) -> u16 {
        unimplemented!()
    }

    fn cmin(&self, channel: u8, values: u16) -> u16 {
        unimplemented!()
    }

    fn cmax(&self, channel: u8, values: u16) -> u16 {
        unimplemented!()
    }
}
