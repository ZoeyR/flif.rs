#![allow(unused)]
use super::PixelVicinity;
use colors::{Channel, ColorSpace, ColorValue};
use DecodingImage;
use components::transformations::ColorRange;
use FlifInfo;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::rac::{Rac, RacRead};
use std::io::Read;
use numbers::near_zero::NearZeroCoder;
use error::*;
use components::transformations::Transform;

pub struct ManiacTree<'a> {
    update_table: &'a UpdateTable,
    root: Option<ManiacNode<'a>>,
}

pub(crate) fn build_pvec(prediction: ColorValue, pix_vic: &PixelVicinity)
    -> [ColorValue; 10]
{
    let mut pvals = [0; 10];
    let mut i = 0;

    let chan = pix_vic.chan;
    if chan == Channel::Green || chan == Channel::Blue {
        pvals[i] = pix_vic.pixel[Channel::Red];
        i += 1;
    }

    if chan == Channel::Blue {
        pvals[i] = pix_vic.pixel[Channel::Green];
        i += 1;
    }

    if chan != Channel::Alpha && pix_vic.is_rgba {
        pvals[i] = pix_vic.pixel[Channel::Alpha];
        i += 1;
    }

    pvals[i] = prediction;

    let left = pix_vic.left.unwrap_or(0);
    let top = pix_vic.top.unwrap_or(0);
    let top_left = pix_vic.top_left.unwrap_or(0);

    // median index
    pvals[i+1] = match prediction {
        pred if pred == left + top - top_left => 0,
        pred if pred == left => 1,
        pred if pred == top => 2,
        _ => 0,
    };

    if let Some(top_left) = pix_vic.top_left {
        pvals[i+2] = left - top_left;
        pvals[i+3] = top_left - top;
    }

    if let Some(top_right) = pix_vic.top_right {
        pvals[i+4] = top - top_right;
    }

    if let Some(top2) = pix_vic.top2 {
        pvals[i+5] = top2 - top;
    }

    if let Some(left2) = pix_vic.left2 {
        pvals[i+6] = left2 - left;
    }

    pvals
}

impl<'a> ManiacTree<'a> {
    pub fn new<R: Read>(
        rac: &mut Rac<R>,
        channel: Channel,
        info: &FlifInfo,
        update_table: &'a UpdateTable,
    ) -> Result<ManiacTree<'a>> {
        let context_a = ChanceTable::new(update_table);
        let context_b = ChanceTable::new(update_table);
        let context_c = ChanceTable::new(update_table);

        let prange = Self::build_prange_vec(channel, info);
        let root = Self::get_node(
            rac,
            &mut [context_a, context_b, context_c],
            update_table,
            &prange,
            info,
        )?;

        Ok(ManiacTree {
            update_table,
            root: Some(root),
        })
    }

    pub fn size(&self) -> usize {
        self.root.as_ref().unwrap().size()
    }

    pub fn depth(&self) -> usize {
        self.root.as_ref().unwrap().depth()
    }

    pub fn process<R: Read>(
        &mut self,
        rac: &mut Rac<R>,
        pvec: &[ColorValue],
        guess: ColorValue,
        min: ColorValue,
        max: ColorValue,
    ) -> Result<ColorValue> {
        if min == max {
            return Ok(min);
        }

        let mut root = None;
        ::std::mem::swap(&mut self.root, &mut root);
        let root = root.unwrap();
        let (root, val) = root.apply(rac, pvec, min - guess, max - guess)?;
        self.root = Some(root);

        Ok(val + guess)
    }

    fn get_node<R: Read>(
        rac: &mut Rac<R>,
        context: &mut [ChanceTable; 3],
        update_table: &'a UpdateTable,
        prange: &[ColorRange],
        info: &FlifInfo,
    ) -> Result<ManiacNode<'a>> {
        let chance_table = ChanceTable::new(update_table);
        let mut property = rac.read_near_zero(0, prange.len() as isize, &mut context[0])?;

        if property == 0 {
            return Ok(ManiacNode::Leaf(chance_table));
        }
        property -= 1;

        let counter = rac.read_near_zero(1 as i32, 512 as i32, &mut context[1])?;
        let test_value = rac.read_near_zero(
            prange[property as usize].min,
            prange[property as usize].max - 1,
            &mut context[2],
        )?;

        let mut left_prange = Vec::new();
        left_prange.extend_from_slice(prange);
        left_prange[property as usize].min = test_value + 1;
        let left = Self::get_inactive_node(rac, context, &left_prange, info)?;

        let mut right_prange = Vec::new();
        right_prange.extend_from_slice(prange);
        right_prange[property as usize].max = test_value;
        let right = Self::get_inactive_node(rac, context, &right_prange, info)?;

        Ok(ManiacNode::Property {
            id: property,
            value: test_value,
            counter: counter as u32,
            table: chance_table,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn get_inactive_node<R: Read>(
        rac: &mut Rac<R>,
        context: &mut [ChanceTable; 3],
        prange: &[ColorRange],
        info: &FlifInfo,
    ) -> Result<InactiveManiacNode> {
        let mut property = rac.read_near_zero(0, prange.len() as isize, &mut context[0])?;

        if property == 0 {
            return Ok(InactiveManiacNode::InactiveLeaf);
        }
        property -= 1;

        let counter = rac.read_near_zero(1 as i32, 512 as i32, &mut context[1])?;
        let test_value = rac.read_near_zero(
            prange[property as usize].min,
            prange[property as usize].max - 1,
            &mut context[2],
        )?;

        let mut left_prange = Vec::new();
        left_prange.extend_from_slice(&prange);
        left_prange[property as usize].min = test_value + 1;
        let left = Self::get_inactive_node(rac, context, &left_prange, info)?;

        let mut right_prange = Vec::new();
        right_prange.extend_from_slice(&prange);
        right_prange[property as usize].max = test_value;
        let right = Self::get_inactive_node(rac, context, &right_prange, info)?;

        Ok(InactiveManiacNode::InactiveProperty {
            id: property,
            value: test_value,
            counter: counter as u32,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn build_prange_vec(channel: Channel, info: &FlifInfo) -> Vec<ColorRange> {
        let mut prange = Vec::new();

        let transform = &info.transform;

        if channel == Channel::Green || channel == Channel::Blue {
            prange.push(transform.range(Channel::Red));
        }

        if channel == Channel::Blue {
            prange.push(transform.range(Channel::Green));
        }

        if channel != Channel::Alpha && info.header.channels == ColorSpace::RGBA {
            prange.push(transform.range(Channel::Alpha));
        }

        prange.push(transform.range(channel));
        prange.push(ColorRange { min: 0, max: 2 });

        let maxdiff = ColorRange {
            min: transform.range(channel).min - transform.range(channel).max,
            max: transform.range(channel).max - transform.range(channel).min,
        };
        prange.push(maxdiff);
        prange.push(maxdiff);
        prange.push(maxdiff);
        prange.push(maxdiff);
        prange.push(maxdiff);

        prange
    }
}

enum ManiacNode<'a> {
    /// Denotes a property node, property nodes are nodes that currently act as leaf nodes but will become inner nodes when their counter reaches zero
    Property {
        id: isize,
        value: i16,
        table: ChanceTable<'a>,
        counter: u32,
        left: Box<InactiveManiacNode>,
        right: Box<InactiveManiacNode>,
    },
    /// Inner nodes are property nodes whose counters have reached zero. They no longer have a context associated with them.
    Inner {
        id: isize,
        value: i16,
        left: Box<ManiacNode<'a>>,
        right: Box<ManiacNode<'a>>,
    },
    /// Leaf nodes are nodes that can never become inner nodes
    Leaf(ChanceTable<'a>),
}

enum InactiveManiacNode {
    InactiveProperty {
        id: isize,
        value: i16,
        counter: u32,
        left: Box<InactiveManiacNode>,
        right: Box<InactiveManiacNode>,
    },
    InactiveLeaf,
}

impl<'a> ManiacNode<'a> {
    // return type is temporary, will be some reasonable pixel value
    pub fn apply<R: Read>(
        self,
        rac: &mut Rac<R>,
        pvec: &[ColorValue],
        min: ColorValue,
        max: ColorValue,
    ) -> Result<(Self, ColorValue)> {
        use self::ManiacNode::*;
        match self {
            Property {
                id,
                value,
                left,
                right,
                mut counter,
                mut table,
            } => {
                if counter > 0 {
                    let val = rac.read_near_zero(min, max, &mut table)?;
                    counter -= 1;
                    Ok((
                        Property {
                            id,
                            value,
                            counter,
                            left,
                            right,
                            table,
                        },
                        val,
                    ))
                } else {
                    let mut left_table = table.clone();
                    let mut right_table = table;

                    let val = if pvec[id as usize] > value {
                        rac.read_near_zero(min, max, &mut left_table)?
                    } else {
                        rac.read_near_zero(min, max, &mut right_table)?
                    };

                    let mut left = Box::new(left.activate(left_table));
                    let mut right = Box::new(right.activate(right_table));
                    Ok((
                        Inner {
                            id,
                            value,
                            left,
                            right,
                        },
                        val,
                    ))
                }
            }
            Inner {
                id,
                value,
                left,
                right,
            } => {
                if pvec[id as usize] > value {
                    let (new_left, val) = left.apply(rac, pvec, min, max)?;
                    Ok((
                        Inner {
                            id,
                            value,
                            left: Box::new(new_left),
                            right,
                        },
                        val,
                    ))
                } else {
                    let (new_right, val) = right.apply(rac, pvec, min, max)?;
                    Ok((
                        Inner {
                            id,
                            value,
                            left,
                            right: Box::new(new_right),
                        },
                        val,
                    ))
                }
            }
            Leaf(mut table) => {
                let val = rac.read_near_zero(min, max, &mut table)?;
                Ok((Leaf(table), val))
            }
        }
    }

    pub fn size(&self) -> usize {
        use self::ManiacNode::*;
        match *self {
            Property {
                ref left,
                ref right,
                ..
            } => 1 + left.size() + right.size(),
            Inner {
                ref left,
                ref right,
                ..
            } => 1 + left.size() + right.size(),
            Leaf(_) => 1,
        }
    }

    pub fn depth(&self) -> usize {
        use self::ManiacNode::*;
        match *self {
            Property {
                ref left,
                ref right,
                ..
            } => 1 + left.size().max(right.size()),
            Inner {
                ref left,
                ref right,
                ..
            } => 1 + left.size().max(right.size()),
            Leaf(_) => 1,
        }
    }
}

impl InactiveManiacNode {
    pub fn activate(self, table: ChanceTable) -> ManiacNode {
        use self::InactiveManiacNode::*;
        use self::ManiacNode::*;
        match self {
            InactiveLeaf => Leaf(table),
            InactiveProperty {
                id,
                value,
                counter,
                left,
                right,
            } => Property {
                id,
                value,
                counter,
                table,
                left,
                right,
            },
        }
    }

    pub fn size(&self) -> usize {
        use self::InactiveManiacNode::*;
        match *self {
            InactiveProperty {
                ref left,
                ref right,
                ..
            } => 1 + left.size() + right.size(),
            InactiveLeaf => 1,
        }
    }

    pub fn depth(&self) -> usize {
        use self::InactiveManiacNode::*;
        match *self {
            InactiveProperty {
                ref left,
                ref right,
                ..
            } => 1 + left.size().max(right.size()),
            InactiveLeaf => 1,
        }
    }
}
