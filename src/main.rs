use std::cmp::{min, max};
use image::{ DynamicImage, Pixel };
use image::error::ImageResult;

enum Quadtree {
    Leaf(i32),
    Branch(Box<Quadtree>, Box<Quadtree>, Box<Quadtree>, Box<Quadtree>)
}

impl Quadtree {
    fn min(&self) -> i32 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Branch(a, b, c, d) => min(min(a.min(), b.min()), min(c.min(), d.min())),
        }
    }
    fn max(&self) -> i32 {
        match self {
            Quadtree::Leaf(value) => *value,
            Quadtree::Branch(a, b, c, d) => max(max(a.max(), b.max()), max(c.max(), d.max())),
        }
    }
    fn contrast(&self) -> i32 {
        match self {
            Quadtree::Leaf(_) => 0,
            Quadtree::Branch(_, _, _, _) => self.max() - self.min(),
        }
    }
    fn serialize(&self, buffer: &mut Vec<i32>) {
        match self {
            Quadtree::Leaf(value) => {
                buffer.push(0i32);
                buffer.push(*value);
            },
            Quadtree::Branch(a, b, c, d) => {
                buffer.push(1i32);
                a.serialize(buffer);
                b.serialize(buffer);
                c.serialize(buffer);
                d.serialize(buffer);
            },
        }
    }
    fn construct(root: &Vec<i32>) -> Box<Quadtree> {
        fn build(data: &Vec<i32>, p: (usize, usize), window: usize) -> Box<Quadtree> {
            if window == 1 {
                return Box::new(Quadtree::Leaf(data[p.0 + p.1*window]))
            }
            let s = window/2;
            return Box::new(Quadtree::Branch(
                build(data, (p.0, p.1), s),
                build(data, (p.0+s, p.1), s),
                build(data, (p.0+s, p.1+s), s),
                build(data, (p.0, p.1+s), s),
            ));
        }
        let rank = (root.len() as f32).sqrt() as usize;
        assert!(root.len() == rank * rank);
        return build(root, (0, 0), rank);
    }
    fn describe(&self, depth: usize) {
        match self {
            Quadtree::Leaf(value) => {
                print!("{} ", value);
            },
            Quadtree::Branch(a, b, c, d) => {
                println!("");
                print!("{} quad {}-{} : ", " ".repeat(depth * 2), self.min(), self.max());
                a.describe(depth+1);
                b.describe(depth+1);
                c.describe(depth+1);
                d.describe(depth+1);
            },
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

    println!("contrast: {}", x.contrast());
    println!("min: {}", x.min());
    println!("max: {}", x.max());
    //x.describe(0);

    // println!("");
    // println!("serialization:");
    // let mut data = Vec::new();
    // x.serialize(&mut data);
    // for item in data.iter() {
    //     print!("{}", item);
    // }
    // println!("");
}
