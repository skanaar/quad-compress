use std::cmp::{min, max};

pub type BitmapData<'a> = &'a Vec<u8>;

pub fn range(a: &u8, b: &u8, c: &u8, d: &u8) -> u8 {
    return max(max(a, b), max(c, d)) - min(min(a, b), min(c, d));
}

pub fn lerp(a: u8, b: u8, factor: f32) -> u8 {
    return ((a as f32) * (1f32 - factor) + (b as f32) * (factor)) as u8;
}

pub type Quad = (u8, u8, u8, u8);
pub struct QuadMeta { pub low: u8, pub average: u8, pub high: u8, pub size: usize  }

pub enum Quadtree {
    Leaf(u8, u8, u8, u8),
    Branch(Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Quad, QuadMeta),
}

pub type Point = (usize, usize);

impl Quadtree {
    pub fn new(pixels: BitmapData) -> Box<Quadtree> {
        let rank = (pixels.len() as f32).sqrt() as usize;
        assert!(pixels.len() as usize == rank * rank);
        return Quadtree::build(pixels, rank, (0,0), rank);
    }
    fn build(pixels: BitmapData, rank: usize, (x, y): Point, size: usize) -> Box<Quadtree> {
        if size == 2 {
            return Box::new(Quadtree::Leaf(
                pixels[x + y*rank],
                pixels[x+1 + y*rank],
                pixels[x + (y+1)*rank],
                pixels[x+1 + (y+1)*rank]
            ))
        }
        let s = size / 2;
        let a = Quadtree::build(pixels, rank, (x, y), s);
        let b = Quadtree::build(pixels, rank, (x+s, y), s);
        let c = Quadtree::build(pixels, rank, (x, y+s), s);
        let d = Quadtree::build(pixels, rank, (x+s, y+s), s);
        let quad = (
            pixels[x + y*rank],
            pixels[x+size-1 + y*rank],
            pixels[x + (y+size-1)*rank],
            pixels[x+size-1 + (y+size-1)*rank]
        );
        let low = min(min(a.low(), b.low()), min(c.low(), d.low()));
        let high = max(max(a.high(), b.high()), max(c.high(), d.high()));
        let aver = average(quad.0, quad.1, quad.2, quad.3);
        let meta = QuadMeta{ low, high, average: aver, size };
        return Box::new(Quadtree::Branch(a, b, c, d, quad, meta));
    }
    #[allow(dead_code)]
    pub fn get(&self, p: Point) -> u8 {
        return self.get_deep(p, 0, (0, 0));
    }
    pub fn get_approx(&self, p: Point, cutoff: u8) -> u8 {
        return self.get_deep(p, cutoff, (0, 0));
    }
    pub fn low(&self) -> u8 {
        return match self {
            Quadtree::Leaf(a, b, c, d) =>  min(min(*a, *b), min(*c, *d)),
            Quadtree::Branch(_, _, _, _, _, meta) => meta.low,
        }
    }
    pub fn high(&self) -> u8 {
        return match self {
            Quadtree::Leaf(a, b, c, d) =>  max(max(*a, *b), max(*c, *d)),
            Quadtree::Branch(_, _, _, _, _, meta) => meta.high,
        }
    }
    pub fn average(&self) -> u8 {
        return match self {
            Quadtree::Leaf(a, b, c, d) =>  average(*a, *b, *c, *d),
            Quadtree::Branch(_, _, _, _, _, meta) => meta.average,
        }
    }
    pub fn get_deep(&self, p: Point, cutoff: u8, self_offset: Point) -> u8 {
        let (x, y) = p;
        let (xo, yo) = self_offset;
        match self {
            Quadtree::Leaf(a, b, c, d) => {
                let contrast = range(a, b, c, d);
                if contrast < cutoff {
                    return average(*a, *b, *c, *d);
                }
                match (x == xo, y == yo) {
                    (true, true) => *a,
                    (false, true) => *b,
                    (true, false) => *c,
                    (false, false) => *d,
                }
            },
            Quadtree::Branch(a, b, c, d, (a_val, b_val, c_val, d_val), meta) => {
                let QuadMeta { size, low, high, .. } = meta;
                let contrast = high - low;
                if contrast < cutoff {
                    let x_coord = (x-xo) as f32 / (*size as f32);
                    let y_coord = (y-yo) as f32 / (*size as f32);
                    let output = lerp(
                        lerp(*a_val, *b_val, x_coord),
                        lerp(*c_val, *d_val, x_coord),
                        y_coord
                    );
                    return if x == xo || y == yo { output/2 } else { output }
                }
                let s = size / 2;
                let left = (x - xo) < s;
                let top = (y - yo) < s;
                return match (left, top) {
                    (true, true) => a.get_deep(p, cutoff, (xo, yo)),
                    (false, true) => b.get_deep(p, cutoff, (xo+s, yo)),
                    (true, false) => c.get_deep(p, cutoff, (xo, yo+s)),
                    (false, false) => d.get_deep(p, cutoff, (xo+s, yo+s)),
                }
            },
        }
    }
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
            Quadtree::Leaf(..) => assert!(false),
            Quadtree::Branch(a, b, c, d, (a_val, b_val, c_val, d_val), _) => {
                assert_eq!(a_val, 1);
                assert_eq!(b_val, 255);
                assert_eq!(c_val, 0);
                assert_eq!(d_val, 4);
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
            Quadtree::Branch(..) => assert!(false),
        }
    }
}

fn average(a: u8, b: u8, c: u8, d: u8) -> u8 {
    return ((a as u16 + b as u16 + c as u16 + d as u16) / 4) as u8;
}
