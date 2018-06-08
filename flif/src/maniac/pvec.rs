use colors::{Channel, ColorSpace, ColorValue};
use decoding_image::{CorePixelVicinity, EdgePixelVicinity};

type Pvec = [ColorValue; 11];

pub(crate) fn core_pvec(pred: ColorValue, pvic: &CorePixelVicinity) -> Pvec {
    let mut pvec = [0; 11];
    let mut i = 0;

    let chan = pvic.chan;
    if chan == Channel::Green || chan == Channel::Blue {
        pvec[i] = pvic.pixel[Channel::Red];
        i += 1;
    }

    if chan == Channel::Blue {
        pvec[i] = pvic.pixel[Channel::Green];
        i += 1;
    }

    if chan != Channel::Alpha && pvic.is_rgba {
        pvec[i] = pvic.pixel[Channel::Alpha];
        i += 1;
    }

    pvec[i] = pred;

    let left = pvic.left;
    let top = pvic.top;
    let top_left = pvic.top_left;

    // median index
    pvec[i+1] = match pred {
        pred if pred == left + top - top_left => 0,
        pred if pred == left => 1,
        pred if pred == top => 2,
        _ => 0,
    };

    pvec[i+2] = left - top_left;
    pvec[i+3] = top_left - top;
    pvec[i+4] = top - pvic.top_right;
    pvec[i+5] = pvic.top2 - top;
    pvec[i+6] = pvic.left2 - left;

    pvec
}

pub(crate) fn edge_pvec(pred: ColorValue, pvic: &EdgePixelVicinity) -> Pvec {
    let mut pvec = [0; 11];
    let mut i = 0;

    let chan = pvic.chan;
    if chan == Channel::Green || chan == Channel::Blue {
        pvec[i] = pvic.pixel[Channel::Red];
        i += 1;
    }

    if chan == Channel::Blue {
        pvec[i] = pvic.pixel[Channel::Green];
        i += 1;
    }

    if chan != Channel::Alpha && pvic.is_rgba {
        pvec[i] = pvic.pixel[Channel::Alpha];
        i += 1;
    }

    pvec[i] = pred;

    let left = pvic.left.unwrap_or(0);
    let top = pvic.top.unwrap_or(0);
    let top_left = pvic.top_left.unwrap_or(0);

    // median index
    pvec[i+1] = match pred {
        pred if pred == left + top - top_left => 0,
        pred if pred == left => 1,
        pred if pred == top => 2,
        _ => 0,
    };

    if let Some(top_left) = pvic.top_left {
        pvec[i+2] = left - top_left;
        pvec[i+3] = top_left - top;
    }

    if let Some(top_right) = pvic.top_right {
        pvec[i+4] = top - top_right;
    }

    if let Some(top2) = pvic.top2 {
        pvec[i+5] = top2 - top;
    }

    if let Some(left2) = pvic.left2 {
        pvec[i+6] = left2 - left;
    }

    pvec
}
