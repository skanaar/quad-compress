use bitvec::prelude::Local;
use bitvec::vec::BitVec;
use crate::quadtree::Quadtree;

pub fn build_leaf_index(quadtree: &Quadtree, quad_index: &mut BitVec<Local, u8>, cutoff: u8) {
    match quadtree {
        Quadtree::Leaf(_, _, _, _) => {
            quad_index.push(false);
        },
        Quadtree::Branch(min, max, a, b, c, d, _) => {
            let contrast = max - min;
            if contrast < cutoff {
                quad_index.push(false);
            } else {
                quad_index.push(true);
                build_leaf_index(a, quad_index, cutoff);
                build_leaf_index(b, quad_index, cutoff);
                build_leaf_index(c, quad_index, cutoff);
                build_leaf_index(d, quad_index, cutoff);
            }
        },
    }
}

pub fn build_leaf_data(quadtree: &Quadtree, leaf_data: &mut Vec<u8>, cutoff: u8) {
    match quadtree {
        Quadtree::Leaf(a, b, c, d) => {
            leaf_data.push(*a);
            leaf_data.push(*b);
            leaf_data.push(*c);
            leaf_data.push(*d);
        },
        Quadtree::Branch(min, max, a, b, c, d, _) => {
            let contrast = max - min;
            if contrast < cutoff {
                leaf_data.push(quadtree.average());
            } else {
                build_leaf_data(a, leaf_data, cutoff);
                build_leaf_data(b, leaf_data, cutoff);
                build_leaf_data(c, leaf_data, cutoff);
                build_leaf_data(d, leaf_data, cutoff);
            }
        },
    }
}
