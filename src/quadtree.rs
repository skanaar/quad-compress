use bitvec::prelude::Local;
use std::cmp::{min, max};
use bitvec::vec::BitVec;

pub type BitmapData<'a> = &'a Vec<u8>;
pub enum Quadtree {
    Leaf(u8),
    Quad(u8, u8, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>),
}

pub type Point = (u32, u32);

fn low_bound(a: u8, b: u8, c: u8, d: u8) -> u8 {
    return min(min(a, b), min(c, d));
}

fn high_bound(a: u8, b: u8, c: u8, d: u8) -> u8 {
    return max(max(a, b), max(c, d));
}

fn lerp(a: u8, b: u8, k: f32) -> u8 {
    return ((a as f32) * (1f32-k) + (b as f32) * (k)) as u8;
}

impl Quadtree {
    pub fn build(pixels: &BitmapData) -> Box<Quadtree> {
        let rank = (pixels.len() as f32).sqrt() as u32;
        assert!(pixels.len() as u32 == rank * rank);
        return Quadtree::build_part(pixels, rank, (0,0), rank);
    }
    fn build_part(pixels: &BitmapData, rank: u32, p: Point, window: u32) -> Box<Quadtree> {
            if window == 1 {
                let pixel = pixels[(p.0 + p.1*rank) as usize];
                return Box::new(Quadtree::Leaf(pixel))
            }
            let s = window / 2;
            let a = Quadtree::build_part(pixels, rank, (p.0, p.1), s);
            let b = Quadtree::build_part(pixels, rank, (p.0+s, p.1), s);
            let c = Quadtree::build_part(pixels, rank, (p.0, p.1+s), s);
            let d = Quadtree::build_part(pixels, rank, (p.0+s, p.1+s), s);
            return Box::new(Quadtree::Quad(
                low_bound((*a).min(), (*b).min(), (*c).min(), (*d).min()),
                high_bound((*a).max(), (*b).max(), (*c).max(), (*d).max()),
                a, b, c, d,
            ));
        }
    pub fn min(&self) -> u8 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(minimum, _, _, _, _, _) => *minimum,
        }
    }
    pub fn max(&self) -> u8 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(_, maximum, _, _, _, _) => *maximum,
        }
    }
    pub fn get(&self, p: Point, cutoff: u8, w: Point, window: u32) -> u8 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(min, max, a, b, c, d) => {
                let contrast = max - min;
                if contrast < cutoff {
                    //return channel(max, req.chan)/2 + channel(min, req.chan)/2;
                    let uncompressed = 0;
                    let d = window-1;
                    let tl = self.get(p, uncompressed, w, window);
                    let tr = self.get((p.0+d, p.1), uncompressed, w, window);
                    let bl = self.get((p.0, p.1+d), uncompressed, w, window);
                    let br = self.get((p.0+d, p.1+d), uncompressed, w, window);
                    return lerp(lerp(tl, tr, 0.5), lerp(bl, br, 0.5), 0.5);
                    //let dx = ((p.0 - w.0) as f32) / (window as f32);
                    //let dy = ((p.1 - w.1) as f32) / (window as f32);
                    //return lerp(lerp(tl, tr, dx), lerp(bl, br, dx), dy);
                }
                let s = window/2;
                let left = (p.0 - w.0) < s;
                let top = (p.1 - w.1) < s;
                return match (left, top) {
                    (true, true) => a.get(p, cutoff, (w.0, w.1), s),
                    (false, true) => b.get(p, cutoff, (w.0+s, w.1), s),
                    (true, false) => c.get(p, cutoff, (w.0, w.1+s), s),
                    (false, false) => d.get(p, cutoff, (w.0+s, w.1+s), s),
                }
            },
        }
    }
    pub fn build_leaf_index(&self, quad_index: &mut BitVec<Local, u8>, cutoff: u8) {
        match self {
            Quadtree::Leaf(_) => {
                quad_index.push(false);
            },
            Quadtree::Quad(min, max, a, b, c, d) => {
                let contrast = max - min;
                if contrast < cutoff {
                    quad_index.push(false);
                } else {
                    quad_index.push(true);
                    a.build_leaf_index(quad_index, cutoff);
                    b.build_leaf_index(quad_index, cutoff);
                    c.build_leaf_index(quad_index, cutoff);
                    d.build_leaf_index(quad_index, cutoff);
                }
            },
        }
    }
    pub fn build_leaf_data(&self, leaf_data: &mut Vec<u8>, cutoff: u8) {
        match self {
            Quadtree::Leaf(value) => leaf_data.push(*value),
            Quadtree::Quad(min, max, a, b, c, d) => {
                let contrast = max - min;
                if contrast < cutoff {
                    let average = min/2 + max/2;
                    leaf_data.push(average);
                } else {
                    a.build_leaf_data(leaf_data, cutoff);
                    b.build_leaf_data(leaf_data, cutoff);
                    c.build_leaf_data(leaf_data, cutoff);
                    d.build_leaf_data(leaf_data, cutoff);
                }
            },
        }
    }
}
