use super::Transformation;

#[derive(Debug)]
pub struct YCoGg {
    max: i16,
}

impl YCoGg {
    pub fn new<T: ?Sized + Transformation>(transformation: &T) -> YCoGg {
        let max_iter = [
            transformation.max(0),
            transformation.max(1),
            transformation.max(2),
        ];

        let old_max = max_iter.iter().max().unwrap();
        let new_max = (((old_max / 4) + 1) * 4) - 1;
        YCoGg { max: new_max }
    }
}

impl Transformation for YCoGg {
    fn snap(&self, _channel: u8, _values: i16, _pixel: i16) -> i16 {
        unimplemented!()
    }

    fn min(&self, channel: u8) -> i16 {
        match channel {
            0 => 0,
            _ => -self.max,
        }
    }

    fn max(&self, _channel: u8) -> i16 {
        self.max
    }

    fn cmin(&self, _channel: u8, _values: i16) -> i16 {
        unimplemented!()
    }

    fn cmax(&self, _channel: u8, _values: i16) -> i16 {
        unimplemented!()
    }
}
