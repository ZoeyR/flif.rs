mod bounds;
mod channel_compact;
mod permute_planes;
mod ycocg;

pub trait Transformation {
    fn snap(&self, channel: u8, values: u16, pixel: u16) -> u16;

    fn min(&self, channel: u8) -> u16;

    fn max(&self, channel: u8) -> u16;

    fn cmin(&self, channel: u8, values: u16) -> u16;

    fn cmax(&self, channel: u8, values: u16) -> u16;
}
