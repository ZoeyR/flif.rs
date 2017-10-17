use super::Transformation;

pub struct YCoGg;

impl YCoGg {
    pub fn new<T: Transformation>(transformation: &T) -> YCoGg {
        YCoGg
    }
}

impl Transformation for YCoGg {
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
