use std::cmp::{min, max};

pub type BitmapData<'a> = &'a Vec<u8>;



pub enum Quadtree {
    Leaf(u8, u8, u8, u8),
    Branch(u8, u8, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, usize),
}

pub type Point = (usize, usize);

impl Quadtree {
    pub fn new(pixels: BitmapData) -> Box<Quadtree> {
        let rank = (pixels.len() as f32).sqrt() as usize;
        assert!(pixels.len() as usize == rank * rank);
        return Quadtree::build(pixels, rank, (0,0), rank);
    }
    fn build(pixels: BitmapData, rank: usize, p: Point, window: usize) -> Box<Quadtree> {
        if window == 2 {
            return Box::new(Quadtree::Leaf(
                pixels[(p.0) + (p.1*rank)],
                pixels[(p.0+1) + (p.1*rank)],
                pixels[(p.0) + (p.1*rank+1)],
                pixels[(p.0+1) + (p.1*rank+1)]
            ))
        }
        let s = window / 2;
        let a = Quadtree::build(pixels, rank, (p.0, p.1), s);
        let b = Quadtree::build(pixels, rank, (p.0+s, p.1), s);
        let c = Quadtree::build(pixels, rank, (p.0, p.1+s), s);
        let d = Quadtree::build(pixels, rank, (p.0+s, p.1+s), s);
        return Box::new(Quadtree::Branch(
            low_bound(a.min(), b.min(), c.min(), d.min()),
            high_bound(a.max(), b.max(), c.max(), d.max()),
            a, b, c, d,
            rank
        ));
    }
    pub fn min(&self) -> u8 {
        match self {
            Quadtree::Leaf(a, b, c, d) => min(min(*a, *b), min(*c, *d)),
            Quadtree::Branch(minimum, _, _, _, _, _, _) => *minimum,
        }
    }
    pub fn max(&self) -> u8 {
        match self {
            Quadtree::Leaf(a, b, c, d) => max(max(*a, *b), max(*c, *d)),
            Quadtree::Branch(_, maximum, _, _, _, _, _) => *maximum,
        }
    }
    #[allow(dead_code)]
    pub fn get(&self, p: Point) -> u8 {
        match self {
            Quadtree::Leaf(_, _, _, _) => self.get_deep(p, 0, (0, 0), 2),
            Quadtree::Branch(_, _, _, _, _, _, rank) => self.get_deep(p, 0, (0, 0), *rank),
        }
    }
    pub fn get_approx(&self, p: Point, cutoff: u8) -> u8 {
        match self {
            Quadtree::Leaf(_, _, _, _) => self.get_deep(p, cutoff, (0, 0), 2),
            Quadtree::Branch(_, _, _, _, _, _, rank) => self.get_deep(p, cutoff, (0, 0), *rank),
        }
    }
    pub fn average(&self) -> u8 {
        match self {
            Quadtree::Leaf(a, b, c, d) => {
                let sum = *a as u32 + *b as u32 + *c as u32 + *d as u32;
                return (sum / 4) as u8;
            },
            Quadtree::Branch(_, _, a, b, c, d, _) => {
                let sum =
                    a.average() as u32 +
                    b.average() as u32 +
                    c.average() as u32 +
                    d.average() as u32;
                return (sum / 4) as u8;
            },
        }
    }
    pub fn get_deep(&self, p: Point, cutoff: u8, self_offset: Point, window: usize) -> u8 {
        match self {
            Quadtree::Leaf(a, b, c, d) => {
                let (x, y) = p;
                let (xo, yo) = self_offset;
                match (x == xo, y == yo) {
                    (true, true) => *a,
                    (false, true) => *b,
                    (true, false) => *c,
                    (false, false) => *d,
                }
            },
            Quadtree::Branch(min, max, a, b, c, d, _) => {
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
}

fn low_bound(a: u8, b: u8, c: u8, d: u8) -> u8 {
    return min(min(a, b), min(c, d));
}

fn high_bound(a: u8, b: u8, c: u8, d: u8) -> u8 {
    return max(max(a, b), max(c, d));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_by_two_quadtree() {
        let bitmap = vec![1u8, 2u8, 3u8, 4u8];
        let quadtree = Quadtree::new(&&bitmap);
        assert_leaf(quadtree, 1, 2, 3, 4);
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
            Quadtree::Leaf(_, _, _, _) => assert!(false),
            Quadtree::Branch(min, max, a, b, c, d, _) => {
                assert_eq!(min, 0);
                assert_eq!(max, 255);
                assert_leaf(a, 1, 1, 1, 1);
                assert_leaf(b, 255, 255, 255, 255);
                assert_leaf(c, 3, 0, 0, 0);
                assert_leaf(d, 4, 4, 4, 4);
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

    fn assert_leaf(node: Box<Quadtree>, av: u8, bv: u8, cv: u8, dv: u8) {
        match *node {
            Quadtree::Leaf(a, b, c, d) => {
                assert_eq!(a, av);
                assert_eq!(b, bv);
                assert_eq!(c, cv);
                assert_eq!(d, dv);
            },
            Quadtree::Branch(_, _, _, _, _, _, _) => assert!(false),
        }
    }
}
