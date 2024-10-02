use bitvec::prelude::Local;
use std::cmp::{min, max};
use bitvec::vec::BitVec;

pub type BitmapData<'a> = &'a Vec<u8>;
pub enum Quadtree {
    Leaf(u8),
    Quad(u8, u8, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, u32),
}

pub type Point = (u32, u32);

impl Quadtree {
    pub fn new(pixels: &BitmapData) -> Box<Quadtree> {
        let rank = (pixels.len() as f32).sqrt() as u32;
        assert!(pixels.len() as u32 == rank * rank);
        return Quadtree::build(pixels, rank, (0,0), rank);
    }
    fn build(pixels: &BitmapData, rank: u32, p: Point, window: u32) -> Box<Quadtree> {
        if window == 1 {
            let pixel = pixels[(p.0 + p.1*rank) as usize];
            return Box::new(Quadtree::Leaf(pixel))
        }
        let s = window / 2;
        let a = Quadtree::build(pixels, rank, (p.0, p.1), s);
        let b = Quadtree::build(pixels, rank, (p.0+s, p.1), s);
        let c = Quadtree::build(pixels, rank, (p.0, p.1+s), s);
        let d = Quadtree::build(pixels, rank, (p.0+s, p.1+s), s);
        return Box::new(Quadtree::Quad(
            low_bound(a.min(), b.min(), c.min(), d.min()),
            high_bound(a.max(), b.max(), c.max(), d.max()),
            a, b, c, d,
            rank
        ));
    }
    pub fn min(&self) -> u8 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(minimum, _, _, _, _, _, _) => *minimum,
        }
    }
    pub fn max(&self) -> u8 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(_, maximum, _, _, _, _, _) => *maximum,
        }
    }
    pub fn get(&self, p: Point) -> u8 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(_, _, _, _, _, _, rank) => self.get_deep(p, 0, (0, 0), *rank),
        }
    }
    pub fn get_approx(&self, p: Point, cutoff: u8) -> u8 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(_, _, _, _, _, _, rank) => self.get_deep(p, cutoff, (0, 0), *rank),
        }
    }
    pub fn average(&self) -> u8 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(_, _, a, b, c, d, _) => {
                let sum =
                    a.average() as u32 +
                    b.average() as u32 +
                    c.average() as u32 +
                    d.average() as u32;
                return (sum / 4) as u8;
            },
        }
    }
    pub fn get_deep(&self, p: Point, cutoff: u8, self_offset: Point, window: u32) -> u8 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(min, max, a, b, c, d, _) => {
                let contrast = max - min;
                if contrast < cutoff {
                    return self.average();
                }
                let (x, y) = p;
                let (xo, yo) = self_offset;
                let s = window/2;
                let left = (x - xo) < s;
                let top = (y - yo) < s;
                return match (left, top) {
                    (true, true) => a.get_deep(p, cutoff, (xo, yo), s),
                    (false, true) => b.get_deep(p, cutoff, (xo+s, yo), s),
                    (true, false) => c.get_deep(p, cutoff, (xo, yo+s), s),
                    (false, false) => d.get_deep(p, cutoff, (xo+s, yo+s), s),
                }
            },
        }
    }
    pub fn build_leaf_index(&self, quad_index: &mut BitVec<Local, u8>, cutoff: u8) {
        match self {
            Quadtree::Leaf(_) => {
                quad_index.push(false);
            },
            Quadtree::Quad(min, max, a, b, c, d, _) => {
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
            Quadtree::Quad(min, max, a, b, c, d, _) => {
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

fn low_bound(a: u8, b: u8, c: u8, d: u8) -> u8 {
    return min(min(a, b), min(c, d));
}

fn high_bound(a: u8, b: u8, c: u8, d: u8) -> u8 {
    return max(max(a, b), max(c, d));
}

fn lerp(a: u8, b: u8, k: f32) -> u8 {
    return ((a as f32) * (1f32-k) + (b as f32) * (k)) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_by_two_quadtree() {
        let bitmap = vec![1u8, 2u8, 3u8, 4u8];
        let quadtree = Quadtree::new(&&bitmap);
        assert_quad_with_leafs(quadtree, 1, 2, 3, 4);
    }

    #[test]
    fn four_by_four_quadtree() {
        let bitmap = vec![
            1u8, 1u8, 255u8, 255u8,
            1u8, 1u8, 255u8, 255u8,
            3u8, 0u8, 4u8, 4u8,
            0u8, 0u8, 4u8, 4u8
        ];
        let quadtree = Quadtree::new(&&bitmap);
        match *quadtree {
            Quadtree::Leaf(_) => assert!(false),
            Quadtree::Quad(min, max, a, b, c, d, _) => {
                assert_eq!(min, 0);
                assert_eq!(max, 255);
                assert_quad_with_leafs(a, 1, 1, 1, 1);
                assert_quad_with_leafs(b, 255, 255, 255, 255);
                assert_quad_with_leafs(c, 3, 0, 0, 0);
                assert_quad_with_leafs(d, 4, 4, 4, 4);
            },
        }
    }

    #[test]
    fn read_exact() {
        let bitmap = vec![
            1u8, 1u8, 255u8, 255u8,
            1u8, 1u8, 255u8, 255u8,
            3u8, 0u8, 4u8, 4u8,
            0u8, 0u8, 4u8, 4u8
        ];
        let quadtree = Quadtree::new(&&bitmap);
        assert_eq!(quadtree.get((0, 0)), 1);
        assert_eq!(quadtree.get((1, 0)), 1);
        assert_eq!(quadtree.get((3, 0)), 255);
        assert_eq!(quadtree.get((3, 3)), 4);
    }

    #[test]
    fn read_approx() {
        let bitmap = vec![
            5u8, 0u8,
            0u8, 0u8
        ];
        let quadtree = Quadtree::new(&&bitmap);
        assert_eq!(1, quadtree.get_approx((0, 0), 10));
        assert_eq!(1, quadtree.get_approx((1, 0), 10));
        assert_eq!(1, quadtree.get_approx((0, 1), 10));
        assert_eq!(1, quadtree.get_approx((1, 1), 10));
    }

    #[test]
    fn read_big_quad_approx() {
        let bitmap = vec![
            1u8, 3u8, 255u8, 255u8,
            1u8, 3u8, 255u8, 255u8,
            5u8, 0u8, 4u8, 4u8,
            0u8, 0u8, 4u8, 4u8
        ];
        let quadtree = Quadtree::new(&&bitmap);
        assert_eq!(2, quadtree.get_approx((0, 0), 3));
        assert_eq!(255, quadtree.get_approx((2, 0), 128));
        assert_eq!(1, quadtree.get_approx((1, 3), 10));
        assert_eq!(4, quadtree.get_approx((3, 3), 5));
    }

    fn assert_leaf(node: Box<Quadtree>, value: u8) {
        match *node {
            Quadtree::Leaf(x) => assert_eq!(x, value),
            Quadtree::Quad(_, _, _, _, _, _, _) => assert!(false),
        }
    }

    fn assert_quad_with_leafs(quad: Box<Quadtree>, av: u8, bv: u8, cv: u8, dv: u8) {
        match *quad {
            Quadtree::Leaf(_) => assert!(false),
            Quadtree::Quad(low, high, a, b, c, d, _) => {
                assert_eq!(low, min(min(av, bv), min(cv, dv)));
                assert_eq!(high, max(max(av, bv), max(cv, dv)));
                assert_leaf(a, av);
                assert_leaf(b, bv);
                assert_leaf(c, cv);
                assert_leaf(d, dv);
            },
        }
    }
}
