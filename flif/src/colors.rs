#![allow(deprecated)]
// re-export to avoid public API breakage
pub use pixels::{ColorSpace, ColorValue};

use std::ops::{Index, IndexMut};

#[deprecated(since="0.3.1")]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Channel {
    Red = 0,
    Green = 1,
    Blue = 2,
    Alpha = 3,
}

#[deprecated(since="0.3.1")]
pub type Pixel = ChannelSet<ColorValue>;

#[deprecated(since="0.3.1")]
#[derive(Copy, Clone, Debug)]
pub struct ChannelSet<T> {
    red: T,
    green: T,
    blue: T,
    alpha: T,
}

impl<T: Default> Default for ChannelSet<T> {
    fn default() -> Self {
        ChannelSet {
            red: T::default(),
            green: T::default(),
            blue: T::default(),
            alpha: T::default(),
        }
    }
}

impl<T> Index<Channel> for ChannelSet<T> {
    type Output = T;

    fn index(&self, channel: Channel) -> &T {
        match channel {
            Channel::Red => &self.red,
            Channel::Green => &self.green,
            Channel::Blue => &self.blue,
            Channel::Alpha => &self.alpha,
        }
    }
}

impl<T> IndexMut<Channel> for ChannelSet<T> {
    fn index_mut(&mut self, channel: Channel) -> &mut T {
        match channel {
            Channel::Red => &mut self.red,
            Channel::Green => &mut self.green,
            Channel::Blue => &mut self.blue,
            Channel::Alpha => &mut self.alpha,
        }
    }
}

impl IntoIterator for ColorSpace {
    type Item = Channel;
    type IntoIter = IntoChannelIter;

    fn into_iter(self) -> Self::IntoIter {
        let length = match self {
            ColorSpace::Monochrome => 1,
            ColorSpace::RGB => 3,
            ColorSpace::RGBA => 4,
        };

        IntoChannelIter {
            next: Channel::Red,
            length,
        }
    }
}

#[deprecated(since="0.3.1")]
pub struct IntoChannelIter {
    next: Channel,
    length: u8,
}

impl Iterator for IntoChannelIter {
    type Item = Channel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None;
        }

        let retval = self.next;
        self.next = match self.next {
            Channel::Red => Channel::Green,
            Channel::Green => Channel::Blue,
            Channel::Blue => Channel::Alpha,
            _ => Channel::Alpha,
        };

        self.length -= 1;
        Some(retval)
    }
}
