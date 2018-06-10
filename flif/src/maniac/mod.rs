#![allow(unused)]

use std::io::Read;

use colors::{Channel, ColorSpace, ColorValue};
use components::transformations::ColorRange;
use components::transformations::Transform;
use error::*;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::near_zero::NearZeroCoder;
use numbers::rac::{Rac, RacRead};
use DecodingImage;
use FlifInfo;

mod pvec;
pub(crate) use self::pvec::{core_pvec, edge_pvec};

pub struct ManiacTree<'a> {
    update_table: &'a UpdateTable,
    nodes: Vec<ManiacNode<'a>>,
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
        let nodes = Self::create_nodes(
            rac,
            &mut [context_a, context_b, context_c],
            update_table,
            prange,
            info,
        )?;

        Ok(ManiacTree {
            update_table,
            nodes,
        })
    }

    pub fn size(&self) -> usize {
        unimplemented!()
    }

    pub fn depth(&self) -> usize {
        unimplemented!()
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

        let val = self.apply(rac, pvec, min - guess, max - guess)?;
        Ok(val + guess)
    }

    fn create_nodes<R: Read>(
        rac: &mut Rac<R>,
        context: &mut [ChanceTable; 3],
        update_table: &'a UpdateTable,
        prange: Vec<ColorRange>,
        info: &FlifInfo,
    ) -> Result<Vec<ManiacNode<'a>>> {
        use self::ManiacNode::*;

        let mut result_vec = vec![];
        let mut process_stack = vec![(0, prange)];
        loop {
            let (index, prange) = match process_stack.pop() {
                Some(process) => process,
                _ => break,
            };

            let node = if index == 0 {
                Self::create_node(rac, context, update_table, &prange, info)?
            } else {
                Self::create_inner_node(rac, context, &prange, info)?
            };

            if index >= result_vec.len() {
                result_vec.resize(index + 1, ManiacNode::InactiveLeaf);
            }

            let (property, test_value) = match node {
                Property { id, value, .. }
                | InactiveProperty { id, value, .. }
                | Inner { id, value } => (id, value),
                _ => {
                    result_vec[index] = node;
                    continue;
                }
            };

            let mut left_prange = prange.clone();
            left_prange[property as usize].min = test_value + 1;

            let mut right_prange = prange;
            right_prange[property as usize].max = test_value;

            process_stack.push((2 * index + 2, right_prange));
            process_stack.push((2 * index + 1, left_prange));
            result_vec[index] = node;
        }

        Ok(result_vec)
    }

    fn create_node<R: Read>(
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

        Ok(ManiacNode::Property {
            id: property,
            table: chance_table,
            value: test_value,
            counter: counter as u32,
        })
    }

    fn create_inner_node<R: Read>(
        rac: &mut Rac<R>,
        context: &mut [ChanceTable; 3],
        prange: &[ColorRange],
        info: &FlifInfo,
    ) -> Result<ManiacNode<'a>> {
        let mut property = rac.read_near_zero(0, prange.len() as isize, &mut context[0])?;

        if property == 0 {
            return Ok(ManiacNode::InactiveLeaf);
        }
        property -= 1;

        let counter = rac.read_near_zero(1 as i32, 512 as i32, &mut context[1])?;
        let test_value = rac.read_near_zero(
            prange[property as usize].min,
            prange[property as usize].max - 1,
            &mut context[2],
        )?;

        Ok(ManiacNode::InactiveProperty {
            id: property,
            value: test_value,
            counter: counter as u32,
        })
    }

    pub fn apply<R: Read>(
        &mut self,
        rac: &mut Rac<R>,
        pvec: &[ColorValue],
        min: ColorValue,
        max: ColorValue,
    ) -> Result<ColorValue> {
        use self::ManiacNode::*;
        let mut node_index = 0;
        let (val, new_node) = loop {
            let mut node = InactiveLeaf;
            ::std::mem::swap(&mut self.nodes[node_index], &mut node);
            match node {
                Property {
                    id,
                    value,
                    mut counter,
                    mut table,
                } => {
                    if counter > 0 {
                        let val = rac.read_near_zero(min, max, &mut table)?;
                        counter -= 1;
                        break (
                            val,
                            Property {
                                id,
                                value,
                                counter,
                                table,
                            },
                        );
                    } else {
                        let mut left_table = table.clone();
                        let mut right_table = table;

                        let val = if pvec[id as usize] > value {
                            rac.read_near_zero(min, max, &mut left_table)?
                        } else {
                            rac.read_near_zero(min, max, &mut right_table)?
                        };

                        let mut left = InactiveLeaf;
                        let mut right = InactiveLeaf;
                        ::std::mem::swap(&mut self.nodes[2 * node_index + 1], &mut left);
                        ::std::mem::swap(&mut self.nodes[2 * node_index + 2], &mut right);
                        self.nodes[2 * node_index + 1] = left.activate(left_table);
                        self.nodes[2 * node_index + 2] = right.activate(right_table);
                        break (val, Inner { id, value });
                    }
                }
                Inner { id, value } => {
                    ::std::mem::swap(&mut self.nodes[node_index], &mut node);
                    if pvec[id as usize] > value {
                        node_index = 2 * node_index + 1;
                    } else {
                        node_index = 2 * node_index + 2;
                    }
                }
                Leaf(mut table) => {
                    let val = rac.read_near_zero(min, max, &mut table)?;
                    break (val, Leaf(table));
                }
                _ => panic!(),
            }
        };

        self.nodes[node_index] = new_node;
        Ok(val)
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

#[derive(Clone)]
enum ManiacNode<'a> {
    /// Denotes a property node, property nodes are nodes that currently act as leaf nodes but will become inner nodes when their counter reaches zero
    Property {
        id: isize,
        value: i16,
        table: ChanceTable<'a>,
        counter: u32,
    },
    InactiveProperty {
        id: isize,
        value: i16,
        counter: u32,
    },
    /// Inner nodes are property nodes whose counters have reached zero. They no longer have a context associated with them.
    Inner {
        id: isize,
        value: i16,
    },
    /// Leaf nodes are nodes that can never become inner nodes
    Leaf(ChanceTable<'a>),
    InactiveLeaf,
}

impl<'a> ManiacNode<'a> {
    // return type is temporary, will be some reasonable pixel value
    pub fn activate(self, table: ChanceTable<'a>) -> Self {
        use self::ManiacNode::*;
        match self {
            InactiveLeaf => Leaf(table),
            InactiveProperty { id, value, counter } => Property {
                id,
                value,
                counter,
                table,
            },
            other => other,
        }
    }
}
