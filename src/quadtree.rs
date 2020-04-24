use std::cmp::{min, max};
use bit_vec::BitVec;

pub type Pix = (u8,u8,u8,u8);
pub struct ImgData<'a> { pub pixels: &'a Vec<Pix>, pub rank: u32 }
pub enum Quadtree {
    Leaf(Pix),
    Quad(Pix, Pix, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>),
}

pub type Cutoff = (u8, u8, u8);

#[derive(Clone, Copy)]
pub struct ChannelReq { pub chan: u8, pub cutoff: u8 }
pub type Point = (u32, u32);

fn channel(pixel: &Pix, i: u8) -> u8 {
    match i {
        0 => (*pixel).0,
        1 => (*pixel).1,
        2 => (*pixel).2,
        3 => (*pixel).3,
        _ => 0,
    }
}

fn low_bound(a: Pix, b: Pix, c: Pix, d: Pix) -> Pix {
    return (
        min(min(a.0, b.0), min(c.0, d.0)),
        min(min(a.1, b.1), min(c.1, d.1)),
        min(min(a.2, b.2), min(c.2, d.2)),
        min(min(a.3, b.3), min(c.3, d.3)),
    );
}

fn lerp(a: u8, b: u8, k: f32) -> u8 {
    return ((a as f32) * (1f32-k) + (b as f32) * (k)) as u8;
}

fn high_bound(a: Pix, b: Pix, c: Pix, d: Pix) -> Pix {
    return (
        max(max(a.0, b.0), max(c.0, d.0)),
        max(max(a.1, b.1), max(c.1, d.1)),
        max(max(a.2, b.2), max(c.2, d.2)),
        max(max(a.3, b.3), max(c.3, d.3)),
    );
}

impl Quadtree {
    pub fn build(data: &ImgData, p: Point, window: u32) -> Box<Quadtree> {
        if window == 1 {
            let pixel = data.pixels[(p.0 + p.1*data.rank) as usize];
            return Box::new(Quadtree::Leaf(pixel))
        }
        let s = window / 2;
        let a = Quadtree::build(data, (p.0, p.1), s);
        let b = Quadtree::build(data, (p.0+s, p.1), s);
        let c = Quadtree::build(data, (p.0, p.1+s), s);
        let d = Quadtree::build(data, (p.0+s, p.1+s), s);
        return Box::new(Quadtree::Quad(
            low_bound((*a).min(), (*b).min(), (*c).min(), (*d).min()),
            high_bound((*a).max(), (*b).max(), (*c).max(), (*d).max()),
            a, b, c, d,
        ));
    }
    pub fn min(&self) -> Pix {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(minimum, _, _, _, _, _) => *minimum,
        }
    }
    pub fn max(&self) -> Pix {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(_, maximum, _, _, _, _) => *maximum,
        }
    }
    pub fn get(&self, p: Point, req: ChannelReq, w: Point, window: u32) -> u8 {
        match self {
            Quadtree::Leaf(value) => channel(value, req.chan),
            Quadtree::Quad(min, max, a, b, c, d) => {
                let contrast = channel(max, req.chan) - channel(min, req.chan);
                if contrast < req.cutoff {
                    //return channel(max, req.chan)/2 + channel(min, req.chan)/2;
                    let uncompressed = ChannelReq { chan: req.chan, cutoff: 0 };
                    let d = window-1;
                    let tl = self.get(p, uncompressed, w, window);
                    let tr = self.get((p.0+d, p.1), uncompressed, w, window);
                    let bl = self.get((p.0, p.1+d), uncompressed, w, window);
                    let br = self.get((p.0+d, p.1+d), uncompressed, w, window);
                    let dx = ((p.0 - w.0) as f32) / (window as f32);
                    let dy = ((p.1 - w.1) as f32) / (window as f32);
                    return lerp(lerp(tl, tr, dx), lerp(bl, br, dx), dy);
                }
                let s = window/2;
                let left = (p.0 - w.0) < s;
                let top = (p.1 - w.1) < s;
                return match (left, top) {
                    (true, true) => a.get(p, req, (w.0, w.1), s),
                    (false, true) => b.get(p, req, (w.0+s, w.1), s),
                    (true, false) => c.get(p, req, (w.0, w.1+s), s),
                    (false, false) => d.get(p, req, (w.0+s, w.1+s), s),
                }
            },
        }
    }
    pub fn build_leaf_index(&self, quad_index: &mut BitVec, req: ChannelReq) {
        match self {
            Quadtree::Leaf(_) => {
                quad_index.push(false);
            },
            Quadtree::Quad(_, _, a, b, c, d) => {
                quad_index.push(true);
                // TODO: use req.cutoff
                a.build_leaf_index(quad_index, req);
                b.build_leaf_index(quad_index, req);
                c.build_leaf_index(quad_index, req);
                d.build_leaf_index(quad_index, req);
            },
        }
    }
    pub fn build_leaf_data(&self, leaf_data: &mut Vec<u8>, req: ChannelReq) {
        match self {
            Quadtree::Leaf(value) => leaf_data.push(channel(value, req.chan)),
            Quadtree::Quad(_, _, a, b, c, d) => {
                // TODO: use req.cutoff
                a.build_leaf_data(leaf_data, req);
                b.build_leaf_data(leaf_data, req);
                c.build_leaf_data(leaf_data, req);
                d.build_leaf_data(leaf_data, req);
            },
        }
    }
}
