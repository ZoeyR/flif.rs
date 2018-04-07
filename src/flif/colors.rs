use std::ops::{Index, IndexMut};

pub type ColorValue = i16;

pub struct Pixel {
    red: ColorValue,
    green: ColorValue,
    blue: ColorValue,
    alpha: ColorValue,
}

impl Index<Channel> for Pixel {
    type Output = ColorValue;

    fn index(&self, channel: Channel) -> &ColorValue {
        match channel {
            Channel::Red => &self.red,
            Channel::Green => &self.green,
            Channel::Blue => &self.blue,
            Channel::Alpha => &self.alpha,
        }
    }
}

impl IndexMut<Channel> for Pixel {
    fn index_mut(&mut self, channel: Channel) -> &mut ColorValue {
        match channel {
            Channel::Red => &mut self.red,
            Channel::Green => &mut self.green,
            Channel::Blue => &mut self.blue,
            Channel::Alpha => &mut self.alpha,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Channel {
    Red,
    Green,
    Blue,
    Alpha
}
