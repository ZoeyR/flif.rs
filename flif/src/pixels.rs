pub type ColorValue = i16;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ColorSpace {
    Monochrome = 1,
    RGB = 3,
    RGBA = 4,
}

pub trait ChannelsTrait {
    fn as_channel(&self) -> RgbaChannels;
    fn is_alpha(&self) -> bool;
}

pub trait Pixel: Default + Copy {
    type Channels: ChannelsTrait + Copy;
    type ChanOrder: AsRef<[Self::Channels]>;

    fn is_rgba() -> bool;
    fn get_value(&self, chan: Self::Channels) -> ColorValue;
    fn set_value(&mut self, val: ColorValue, chan: Self::Channels);
    /// Return if alpha channel equals to zero. For non-RGBA images always
    /// returns `false`.
    fn is_alpha_zero(&self) -> bool;

    /// Return red value if chan is green or blue. Always None for greyscale.
    fn get_red_pvec(&self, chan: Self::Channels) -> Option<ColorValue>;
    /// Return green value if chan is blue. Always None for greyscale.
    fn get_green_pvec(&self, chan: Self::Channels) -> Option<ColorValue>;
    /// Return alpha value if chan is not alpha. Always None for non-RGBA.
    fn get_alpha_pvec(&self, chan: Self::Channels) -> Option<ColorValue>;

    fn to_rgba(&self) -> Rgba;
    fn get_chan_order() -> Self::ChanOrder;
    fn size() -> usize;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Greyscale(ColorValue);

#[derive(Debug, Copy, Clone)]
pub enum GreyChannels {
    Grey = 0,
}

impl ChannelsTrait for GreyChannels {
    #[inline(always)]
    fn as_channel(&self) -> RgbaChannels { RgbaChannels::Red }
    #[inline(always)]
    fn is_alpha(&self) -> bool { false }
}

impl Pixel for Greyscale {
    type Channels = GreyChannels;
    type ChanOrder = [GreyChannels; 1];

    #[inline(always)]
    fn is_rgba() -> bool { false }
    #[inline(always)]
    fn get_value(&self, _chan: Self::Channels) -> ColorValue { self.0 }
    #[inline(always)]
    fn set_value(&mut self, val: ColorValue, _chan: Self::Channels) {
        self.0 = val;
    }
    #[inline(always)]
    fn is_alpha_zero(&self) -> bool { false }
    #[inline(always)]
    fn get_red_pvec(&self, _chan: Self::Channels) -> Option<ColorValue> { None }
    #[inline(always)]
    fn get_green_pvec(&self, _chan: Self::Channels) -> Option<ColorValue> {
        None
    }
    #[inline(always)]
    fn get_alpha_pvec(&self, _chan: Self::Channels) -> Option<ColorValue> {
        None
    }
    #[inline(always)]
    fn to_rgba(&self) -> Rgba { Rgba([self.0, 0, 0, 0]) }
    #[inline(always)]
    fn get_chan_order() -> Self::ChanOrder { [GreyChannels::Grey] }
    #[inline(always)]
    fn size() -> usize { 1 }
}


#[derive(Debug, Default, Copy, Clone)]
pub struct Rgb([ColorValue; 3]);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RgbChannels {
    Red = 0,
    Green = 1,
    Blue = 2,
}

impl ChannelsTrait for RgbChannels {
    #[inline(always)]
    fn as_channel(&self) -> RgbaChannels {
        match self {
            RgbChannels::Red => RgbaChannels::Red,
            RgbChannels::Green => RgbaChannels::Green,
            RgbChannels::Blue => RgbaChannels::Blue,
        }
    }
    #[inline(always)]
    fn is_alpha(&self) -> bool { false }
}

impl Pixel for Rgb {
    type Channels = RgbChannels;
    type ChanOrder = [RgbChannels; 3];

    #[inline(always)]
    fn is_rgba() -> bool { false }
    #[inline(always)]
    fn get_value(&self, chan: Self::Channels) -> ColorValue {
        self.0[chan as usize]
    }
    #[inline(always)]
    fn set_value(&mut self, val: ColorValue, chan: Self::Channels) {
        self.0[chan as usize] = val;
    }
    #[inline(always)]
    fn is_alpha_zero(&self) -> bool { false }
    #[inline(always)]
    fn get_red_pvec(&self, chan: Self::Channels) -> Option<ColorValue> {
        if chan == RgbChannels::Green || chan == RgbChannels::Blue {
            Some(self.0[0])
        } else {
            None
        }
    }
    #[inline(always)]
    fn get_green_pvec(&self, chan: Self::Channels) -> Option<ColorValue> {
        if chan == RgbChannels::Blue { Some(self.0[1]) } else { None }
    }
    #[inline(always)]
    fn get_alpha_pvec(&self, _chan: Self::Channels) -> Option<ColorValue> {
        None
    }
    #[inline(always)]
    fn to_rgba(&self) -> Rgba { Rgba([self.0[0], self.0[1], self.0[2], 0]) }
    #[inline(always)]
    fn get_chan_order() -> Self::ChanOrder {
        [RgbChannels::Red, RgbChannels::Green, RgbChannels::Blue]
    }
    #[inline(always)]
    fn size() -> usize { 3 }
}


#[derive(Debug, Default, Copy, Clone)]
pub struct Rgba(pub [ColorValue; 4]);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RgbaChannels {
    Red = 0,
    Green = 1,
    Blue = 2,
    Alpha = 3,
}

impl RgbaChannels {
    /// this order is different from `get_chan_order`
    pub const ORDER: [Self; 4] = [
        RgbaChannels::Red, RgbaChannels::Green,
        RgbaChannels::Blue, RgbaChannels::Alpha
    ];
}

impl ChannelsTrait for RgbaChannels {
    #[inline(always)]
    fn as_channel(&self) -> RgbaChannels { *self }
    #[inline(always)]
    fn is_alpha(&self) -> bool { *self == RgbaChannels::Alpha }
}

impl Pixel for Rgba {
    type Channels = RgbaChannels;
    type ChanOrder = [RgbaChannels; 4];

    #[inline(always)]
    fn is_rgba() -> bool { true }
    #[inline(always)]
    fn get_value(&self, chan: Self::Channels) -> ColorValue {
        self.0[chan as usize]
    }
    #[inline(always)]
    fn set_value(&mut self, val: ColorValue, chan: Self::Channels) {
        self.0[chan as usize] = val;
    }
    #[inline(always)]
    fn is_alpha_zero(&self) -> bool { self.0[3] == 0 }
    #[inline(always)]
    fn get_red_pvec(&self, chan: Self::Channels) -> Option<ColorValue> {
        if chan == RgbaChannels::Green || chan == RgbaChannels::Blue {
            Some(self.0[0])
        } else {
            None
        }
    }
    #[inline(always)]
    fn get_green_pvec(&self, chan: Self::Channels) -> Option<ColorValue> {
        if chan == RgbaChannels::Blue { Some(self.0[1]) } else { None }
    }
    #[inline(always)]
    fn get_alpha_pvec(&self, chan: Self::Channels) -> Option<ColorValue> {
        if chan != RgbaChannels::Alpha { Some(self.0[3]) } else { None }
    }
    #[inline(always)]
    fn to_rgba(&self) -> Rgba { *self }
    #[inline(always)]
    fn get_chan_order() -> Self::ChanOrder {
        [
            RgbaChannels::Alpha,
            RgbaChannels::Red,
            RgbaChannels::Green,
            RgbaChannels::Blue,
        ]
    }
    #[inline(always)]
    fn size() -> usize { 4 }
}

