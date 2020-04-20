use std::cmp::{min, max};
use image::{ RgbImage, ImageBuffer, DynamicImage, Pixel };
use image::error::ImageResult;

enum Quadtree {
    Leaf(i32),
    Branch(i32, i32, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>),
    Root(u32, Box<Quadtree>),
}

impl Quadtree {
    fn construct(input: &Vec<i32>) -> Box<Quadtree> {
        let rank = (input.len() as f32).sqrt() as usize;
        fn build(data: &Vec<i32>, p: (usize, usize), window: usize, rank: usize) -> Box<Quadtree> {
            if window == 1 {
                return Box::new(Quadtree::Leaf(data[p.0 + p.1*rank]))
            }
            let s = window/2;
            let a = build(data, (p.0, p.1), s, rank);
            let b = build(data, (p.0+s, p.1), s, rank);
            let c = build(data, (p.0, p.1+s), s, rank);
            let d = build(data, (p.0+s, p.1+s), s, rank);
            return Box::new(Quadtree::Branch(
                min(min((*a).min(), (*b).min()), min((*c).min(), (*d).min())),
                max(max((*a).max(), (*b).max()), max((*c).max(), (*d).max())),
                a, b, c, d,
            ));
        }
        assert!(input.len() == rank * rank);
        return Box::new(Quadtree::Root(rank as u32, build(input, (0, 0), rank, rank)));
    }
    fn min(&self) -> i32 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Branch(min, _, _, _, _, _) => *min,
            Quadtree::Root(_, data) => data.min(),
        }
    }
    fn max(&self) -> i32 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Branch(_, max, _, _, _, _) => *max,
            Quadtree::Root(_, data) => data.max(),
        }
    }
    fn contrast(&self) -> i32 {
        return self.max() - self.min();
    }
}

impl Quadtree {
    fn get(&self, x: u32, y: u32, offset_x: u32, offset_y: u32, window: u32) -> i32 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Branch(min, max, a, b, c, d) => {
                if self.contrast() < 20 {
                    return (max + min) / 2;
                } else {
                    let s = window/2;
                    let left = (x - offset_x) < s;
                    let top = (y - offset_y) < s;
                    match (left, top) {
                        (true, true) => a.get(x, y, offset_x, offset_y, s),
                        (false, true) => b.get(x, y, offset_x +  s, offset_y, s),
                        (true, false) => c.get(x, y, offset_x, offset_y + s, s),
                        (false, false) => d.get(x, y, offset_x + s, offset_y + s, s),
                    }
                }
            },
            Quadtree::Root(_, data) => data.get(x, y, offset_x, offset_y, window),
        }
    }
    fn to_image(&self) -> RgbImage {
        match self {
            Quadtree::Leaf(_) => panic!(),
            Quadtree::Branch(_, _, _, _, _, _) => panic!(),
            Quadtree::Root(rank, data) => ImageBuffer::from_fn(*rank, *rank, |x, y| {
                let c = data.get(x, y, 0, 0, *rank) as u8;
                image::Rgb([c, c, c])
            }),
        }
    }
}

fn analyze_img(img_res: ImageResult<DynamicImage>) -> Box<Quadtree> {
    let rgb = img_res.unwrap().to_rgb();
    let pixels = rgb.pixels();
    let len = pixels.len();
    let mut data = vec![0; len];
    for (i, pixel) in pixels.enumerate() {
        data[i] = pixel.channels4().0 as i32;
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

    println!("contrast: {}", x.contrast());
    println!("min: {}", x.min());
    println!("max: {}", x.max());
}
