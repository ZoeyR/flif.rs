#![allow(unused)]

use components::transformations::ColorRange;
use ::FlifInfo;
use numbers::rac::ChanceTable;
use numbers::rac::{IRac, Rac};
use std::io::Read;
use numbers::near_zero::NearZeroCoder;
use error::*;

pub(crate) struct ManiacTree {
    root: ManiacNode
}

impl ManiacTree {
    pub fn new<R: Read>(rac: &mut Rac<R>, channel: usize, info: &FlifInfo) -> Result<ManiacTree> {
        let context_a = ChanceTable::new(info.second_header.alpha_divisor, info.second_header.cutoff);
        let context_b = ChanceTable::new(info.second_header.alpha_divisor, info.second_header.cutoff);
        let context_c = ChanceTable::new(info.second_header.alpha_divisor, info.second_header.cutoff);

		let prange = Self::build_prange_vec(channel, info);
		let root = Self::get_node(rac, &mut [context_a, context_b, context_c], &prange, info)?;

        Ok(ManiacTree {
			root
		})
    }

	fn get_node<R: Read>(rac: &mut Rac<R>, context: &mut [ChanceTable; 3], prange: &[ColorRange], info: &FlifInfo) -> Result<ManiacNode> {
		let chance_table = ChanceTable::new(info.second_header.alpha_divisor, info.second_header.cutoff);
		let mut property = rac.read_near_zero_2(0, prange.len() as isize, &mut context[0])?;

		if property == 0 {
			return Ok(ManiacNode::Leaf(chance_table))
		}
		property -= 1;

		let counter = rac.read_near_zero_2(1 as i32, 512 as i32, &mut context[1])?;
		let test_value = rac.read_near_zero_2(prange[property as usize].min, prange[property as usize].max - 1, &mut context[2])?;
		
		let mut left_prange = Vec::new();
		left_prange.extend_from_slice(prange);
		left_prange[property as usize].min = test_value + 1;
		let left = Self::get_inactive_node(rac, context, left_prange, info)?;

		let mut right_prange = Vec::new();
		right_prange.extend_from_slice(prange);
		right_prange[property as usize].max = test_value;
		let right = Self::get_inactive_node(rac, context, right_prange, info)?;

		Ok(ManiacNode::Property{id: 0, value: 0, counter: counter as u32, table: chance_table, left: Box::new(left), right: Box::new(right)})
	}

	fn get_inactive_node<R: Read>(rac: &mut Rac<R>, context: &mut [ChanceTable; 3], prange: Vec<ColorRange>, info: &FlifInfo) -> Result<InactiveManiacNode> {
			let mut property = rac.read_near_zero_2(0, prange.len() as isize, &mut context[0])?;

			if property == 0 {
				return Ok(InactiveManiacNode::InactiveLeaf);
			}
			property -= 1;

			let counter = rac.read_near_zero_2(1 as i32, 512 as i32, &mut context[1])?;
			let test_value = rac.read_near_zero_2(prange[property as usize].min, prange[property as usize].max - 1, &mut context[2])?;
		
			let mut left_prange = Vec::new();
			left_prange.extend_from_slice(&prange);
			left_prange[property as usize].min = test_value + 1;
			let left = Self::get_inactive_node(rac, context, left_prange, info)?;

			let mut right_prange = Vec::new();
			right_prange.extend_from_slice(&prange);
			right_prange[property as usize].max = test_value;
			let right = Self::get_inactive_node(rac, context, right_prange, info)?;

			Ok(InactiveManiacNode::InactiveProperty{id: 0, value: 0, counter: counter as u32, left: Box::new(left), right: Box::new(right)})
	}

    fn build_prange_vec(channel: usize, info: &FlifInfo) -> Vec<ColorRange> {
        let mut prange = Vec::new();

		let transform = &info.second_header.transformations;

		if channel > 0 && channel < 3 {
			prange.push(transform.range(0));
		}

		if channel > 1 && channel < 3 {
			prange.push(transform.range(1));
		}

		if channel < 3 && info.header.channels as u8 > 3 {
			prange.push(transform.range(3));
		}

		prange.push(transform.range(channel));
		prange.push(ColorRange {min: 0, max: 2});
		
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

enum ManiacNode {
    
    /// Denotes a property node, property nodes are nodes that currently act as leaf nodes but will become inner nodes when their counter reaches zero
    Property{id: isize, value: i16, table: ChanceTable, counter: u32, left: Box<InactiveManiacNode>, right: Box<InactiveManiacNode>},
    /// Inner nodes are property nodes whose counters have reached zero. They no longer have a context associated with them.
    Inner{id: isize, value: i16, left: Box<ManiacNode>, right: Box<ManiacNode>},
    /// Leaf nodes are nodes that can never become inner nodes
    Leaf(ChanceTable)
}

enum InactiveManiacNode {
    InactiveProperty{id: isize, value: i16, counter: u32, left: Box<InactiveManiacNode>, right: Box<InactiveManiacNode>},
    InactiveLeaf
}

impl ManiacNode {
    // return type is temporary, will be some reasonable pixel value
    pub fn apply<R: Read>(self, rac: &mut Rac<R>, pvec: Vec<i16>, min: i32, max: i32) -> Result<(Self, i32)> {
        use self::ManiacNode::*;
        match self {
            Property{id, value, left, right, mut counter, mut table} => {
                let val = rac.read_near_zero(min, max, &mut table)?;
                counter -= 1;
                if counter == 0 {
                    let left = Box::new(left.activate(table.clone()));
                    let right = Box::new(right.activate(table));
                    Ok((Inner{id, value, left, right}, val))
                } else {
                    Ok((Property{id, value, counter, left, right, table}, val))
                }
            },
            Inner{id, value, left, right} => {
                if pvec[id as usize] > value {
                    left.apply(rac, pvec, min, max)
                } else {
                    right.apply(rac, pvec, min, max)
                }
            },
            Leaf(mut table) => {
                let val = rac.read_near_zero(min, max, &mut table)?;
                Ok((Leaf(table), val))
            },
        }
    }
}

impl InactiveManiacNode {
    pub fn activate(self, table: ChanceTable) -> ManiacNode {
        use self::InactiveManiacNode::*;
        use self::ManiacNode::*;
        match self {
            InactiveLeaf => {
                Leaf(table)
            },
            InactiveProperty{id, value, counter, left, right} => {
                Property{id, value, counter, table, left, right}
            }
        }
    }
}
