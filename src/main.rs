use std::cmp::{min, max};
use image::{ RgbImage, ImageBuffer, DynamicImage, Pixel };
use image::error::ImageResult;

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

enum Quadtree {
    Leaf(Pix),
    Branch(Pix, Pix, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>),
    Root(u32, Box<Quadtree>),
}

impl Quadtree {
    fn construct(input: &Vec<Pix>) -> Box<Quadtree> {
        let rank = (input.len() as f32).sqrt() as usize;
        fn build(data: &Vec<Pix>, p: (usize, usize), window: usize, rank: usize) -> Box<Quadtree> {
            if window == 1 {
                return Box::new(Quadtree::Leaf(data[p.0 + p.1*rank]))
            }
            let s = window/2;
            let a = build(data, (p.0, p.1), s, rank);
            let b = build(data, (p.0+s, p.1), s, rank);
            let c = build(data, (p.0, p.1+s), s, rank);
            let d = build(data, (p.0+s, p.1+s), s, rank);
            return Box::new(Quadtree::Branch(
                low_bound((*a).min(), (*b).min(), (*c).min(), (*d).min()),
                high_bound((*a).max(), (*b).max(), (*c).max(), (*d).max()),
                a, b, c, d,
            ));
        }
        assert!(input.len() == rank * rank);
        return Box::new(Quadtree::Root(rank as u32, build(input, (0, 0), rank, rank)));
    }
    fn min(&self) -> Pix {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Branch(minimum, _, _, _, _, _) => *minimum,
            Quadtree::Root(_, data) => data.min(),
        }
    }
    fn max(&self) -> Pix {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Branch(_, maximum, _, _, _, _) => *maximum,
            Quadtree::Root(_, data) => data.max(),
        }
    }
}

impl Quadtree {
    fn get(&self, component: u8, threshold: u8, x: u32, y: u32, offset_x: u32, offset_y: u32, window: u32) -> u8 {
        match self {
            Quadtree::Leaf(value) => channel(value, component),
            Quadtree::Branch(min, max, a, b, c, d) => {
                if (channel(max, component) - channel(min, component)) < threshold {
                    return channel(max, component)/2 + channel(min, component)/2;
                } else {
                    let s = window/2;
                    let left = (x - offset_x) < s;
                    let top = (y - offset_y) < s;
                    match (left, top) {
                        (true, true) => a.get(component, threshold, x, y, offset_x, offset_y, s),
                        (false, true) => b.get(component, threshold, x, y, offset_x +  s, offset_y, s),
                        (true, false) => c.get(component, threshold, x, y, offset_x, offset_y + s, s),
                        (false, false) => d.get(component, threshold, x, y, offset_x + s, offset_y + s, s),
                    }
                }
            },
            Quadtree::Root(_, data) => data.get(component, threshold, x, y, offset_x, offset_y, window),
        }
    }
    fn to_image(&self) -> RgbImage {
        match self {
            Quadtree::Leaf(_) => panic!(),
            Quadtree::Branch(_, _, _, _, _, _) => panic!(),
            Quadtree::Root(rank, data) => ImageBuffer::from_fn(*rank, *rank, |x, y| {
                let r = data.get(0, 50, x, y, 0, 0, *rank);
                let g = data.get(1, 4, x, y, 0, 0, *rank);
                let b = data.get(2, 100, x, y, 0, 0, *rank);
                image::Rgb([r, g, b])
            }),
        }
    }
}

fn analyze_img(img_res: ImageResult<DynamicImage>) -> Box<Quadtree> {
    let rgb = img_res.unwrap().to_rgb();
    let pixels = rgb.pixels();
    let len = pixels.len();
    let mut data = vec![(0u8,0u8,0u8,0u8); len];
    for (i, pixel) in pixels.enumerate() {
        data[i] = pixel.channels4();
    }
    return Quadtree::construct(&data);
}

fn main() {
    // let v: Vec<i32> = vec![
    //     4, 2, 5, 5,
    //     2, 2, 5, 2,
    //     2, 3, 2, 2,
    //     2, 3, 2, 2
    // ];
    //let x = Quadtree::construct(&v);
    let x = analyze_img(image::open("./lena.png"));
    let res = x.to_image().save("./output.png");
    println!("saved image: {}", res.is_ok());
}
