use std::cmp::{min, max};
use image::{ RgbImage, ImageBuffer, DynamicImage, Pixel };
use image::error::ImageResult;
use bit_vec::BitVec;

type Pix = (u8,u8,u8,u8);

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

fn high_bound(a: Pix, b: Pix, c: Pix, d: Pix) -> Pix {
    return (
        max(max(a.0, b.0), max(c.0, d.0)),
        max(max(a.1, b.1), max(c.1, d.1)),
        max(max(a.2, b.2), max(c.2, d.2)),
        max(max(a.3, b.3), max(c.3, d.3)),
    );
}

type Point = (u32, u32);

struct ImgCompressor {
    root: Box<Quadtree>,
    rank: u32
}

enum Quadtree {
    Leaf(Pix),
    Quad(Pix, Pix, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>),
    Root(u32, Box<Quadtree>),
}

struct ImgData<'a> { pixels: &'a Vec<Pix>, rank: u32 }

impl Quadtree {
    fn build(data: &ImgData, p: Point, window: u32) -> Box<Quadtree> {
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
    fn min(&self) -> Pix {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(minimum, _, _, _, _, _) => *minimum,
            Quadtree::Root(_, data) => data.min(),
        }
    }
    fn max(&self) -> Pix {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Quad(_, maximum, _, _, _, _) => *maximum,
            Quadtree::Root(_, data) => data.max(),
        }
    }
}

struct PixelReq { x: u32, y: u32, chan: u8, cutoff: u8 }

impl Quadtree {
    fn get(&self, req: PixelReq, win_x: u32, win_y: u32, win_size: u32) -> u8 {
        match self {
            Quadtree::Leaf(value) => channel(value, req.chan),
            Quadtree::Quad(min, max, a, b, c, d) => {
                let contrast = channel(max, req.chan) - channel(min, req.chan);
                if contrast < req.cutoff {
                    return channel(max, req.chan)/2 + channel(min, req.chan)/2;
                } else {
                    let s = win_size/2;
                    let left = (req.x - win_x) < s;
                    let top = (req.y - win_y) < s;
                    match (left, top) {
                        (true, true) => a.get(req, win_x, win_y, s),
                        (false, true) => b.get(req, win_x +  s, win_y, s),
                        (true, false) => c.get(req, win_x, win_y + s, s),
                        (false, false) => d.get(req, win_x + s, win_y + s, s),
                    }
                }
            },
            Quadtree::Root(_, data) => data.get(req, win_x, win_y, win_size),
        }
    }
    fn size(&self, cutoffs: (u8, u8, u8)) -> usize {
        match self {
            Quadtree::Leaf(_) => 1,
            Quadtree::Quad(_, _, a, b, c, d) => {
                a.size(cutoffs) +
                b.size(cutoffs) +
                c.size(cutoffs) +
                d.size(cutoffs) + 1
            },
            Quadtree::Root(_, data) => data.size(cutoffs),
        }
    }
    fn build_index(&self, quad_index: &mut BitVec) {
        match self {
            Quadtree::Leaf(_) => {
                quad_index.push(false);
            },
            Quadtree::Quad(_, _, a, b, c, d) => {
                quad_index.push(true);
                a.build_index(quad_index);
                b.build_index(quad_index);
                c.build_index(quad_index);
                d.build_index(quad_index);
            },
            Quadtree::Root(_, _) => {},
        }
    }
    fn build_leaf_data(&self, leaf_data: &mut Vec<Pix>) {
        match self {
            Quadtree::Leaf(value) => leaf_data.push(*value),
            Quadtree::Quad(_, _, a, b, c, d) => {
                a.build_leaf_data(leaf_data);
                b.build_leaf_data(leaf_data);
                c.build_leaf_data(leaf_data);
                d.build_leaf_data(leaf_data);
            },
            Quadtree::Root(_, _) => {},
        }
    }
    fn to_image(&self, cutoffs: (u8, u8, u8)) -> RgbImage {
        match self {
            Quadtree::Leaf(_) => panic!(),
            Quadtree::Quad(_, _, _, _, _, _) => panic!(),
            Quadtree::Root(rank, data) => {
                return ImageBuffer::from_fn(*rank, *rank, |x, y| {
                    let r = PixelReq{ x, y, chan: 0, cutoff: cutoffs.0 };
                    let g = PixelReq{ x, y, chan: 1, cutoff: cutoffs.1 };
                    let b = PixelReq{ x, y, chan: 2, cutoff: cutoffs.2 };
                    image::Rgb([
                        data.get(r, 0, 0, *rank),
                        data.get(g, 0, 0, *rank),
                        data.get(b, 0, 0, *rank),
                    ])
                })
            },
        }
    }
}

impl ImgCompressor {
    fn new(img_res: ImageResult<DynamicImage>) -> ImgCompressor {
        let rgb = img_res.unwrap().to_rgb();
        let pixels = rgb.pixels();
        let len = pixels.len();
        let mut data = vec![(0u8,0u8,0u8,0u8); len];
        for (i, pixel) in pixels.enumerate() {
            data[i] = pixel.channels4();
        }
        return ImgCompressor {
            root: ImgCompressor::build_tree(&data),
            rank: 0,
        };
    }
    fn build_tree(pixels: &Vec<Pix>) -> Box<Quadtree> {
        let rank = (pixels.len() as f32).sqrt() as u32;
        assert!(pixels.len() as u32 == rank * rank);
        let img = ImgData { pixels, rank };
        let quadtree = Quadtree::build(&img, (0, 0), rank);
        return Box::new(Quadtree::Root(rank as u32, quadtree));
    }
}

fn main() {
    let compression = (50, 4, 100);
    let compressor = ImgCompressor::new(image::open("./lena.png"));
    let node_count = compressor.root.size(compression);
    let mut quad_index = BitVec::from_elem(node_count, true);
    let quad_index = compressor.root.build_index(&mut quad_index);
    let mut leaf_data = vec![(0u8,0u8,0u8,0u8); 0];
    let data = compressor.root.build_leaf_data(&mut leaf_data);
    let res = compressor.root.to_image(compression).save("./output.png");
    println!("saved image: {}", res.is_ok());
}
