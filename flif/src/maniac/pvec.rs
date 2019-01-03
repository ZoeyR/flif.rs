use pixels::{Pixel, ColorSpace, ColorValue};
use decoding_image::{CorePixelVicinity, EdgePixelVicinity};

type Pvec = [ColorValue; 10];

pub(crate) fn core_pvec<P: Pixel>(pred: ColorValue, pvic: &CorePixelVicinity<P>) -> Pvec {
    let mut pvec = [0; 10];
    let mut i = 0;

    let chan = pvic.chan;
    if let Some(val) = pvic.pixel.get_red_pvec(chan) {
        pvec[i] = val;
        i += 1;
    }

    if let Some(val) = pvic.pixel.get_green_pvec(chan) {
        pvec[i] = val;
        i += 1;
    }

    if let Some(val) = pvic.pixel.get_alpha_pvec(chan) {
        pvec[i] = val;
        i += 1;
    }

    pvec[i] = pred;

    let left = pvic.left;
    let top = pvic.top;
    let top_left = pvic.top_left;

    // median index
    pvec[i + 1] = match pred {
        pred if pred == left + top - top_left => 0,
        pred if pred == left => 1,
        pred if pred == top => 2,
        _ => 0,
    };

    pvec[i + 2] = left - top_left;
    pvec[i + 3] = top_left - top;
    pvec[i + 4] = top - pvic.top_right;
    pvec[i + 5] = pvic.top2 - top;
    pvec[i + 6] = pvic.left2 - left;

    pvec
}

pub(crate) fn edge_pvec<P: Pixel>(pred: ColorValue, pvic: &EdgePixelVicinity<P>) -> Pvec {
    let mut pvec = [0; 10];
    let mut i = 0;

    let chan = pvic.chan;
    if let Some(val) = pvic.pixel.get_red_pvec(chan) {
        pvec[i] = val;
        i += 1;
    }

    if let Some(val) = pvic.pixel.get_green_pvec(chan) {
        pvec[i] = val;
        i += 1;
    }

    if let Some(val) = pvic.pixel.get_alpha_pvec(chan) {
        pvec[i] = val;
        i += 1;
    }

    pvec[i] = pred;

    // median index
    if let (Some(left), Some(top), Some(top_left)) =
        (pvic.left, pvic.top, pvic.top_left)
    {
        if pred == left + top - top_left { }
        else if pred == left { pvec[i + 1] = 1 }
        else if pred == top { pvec[i + 1] = 2 }
    }

    if let (Some(top_left), Some(left)) = (pvic.top_left, pvic.left) {
        pvec[i + 2] = left - top_left;
    }

    if let (Some(top_left), Some(top)) = (pvic.top_left, pvic.top) {
        pvec[i + 3] = top_left - top;
    }

    if let (Some(top_right), Some(top)) = (pvic.top_right, pvic.top) {
        pvec[i + 4] = top - top_right;
    }

    if let (Some(top2), Some(top)) = (pvic.top2, pvic.top) {
        pvec[i + 5] = top2 - top;
    }

    if let (Some(left2), Some(left)) = (pvic.left2, pvic.left) {
        pvec[i + 6] = left2 - left;
    }

    pvec
}
