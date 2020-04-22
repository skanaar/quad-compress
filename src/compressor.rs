use bit_vec::BitVec;
use image::{ RgbImage, DynamicImage, ImageBuffer, Pixel };
use image::error::ImageResult;

use crate::quadtree::{ Quadtree, Pix, ImgData, PixelReq };

pub struct ImgCompressor {
    pub root: Box<Quadtree>,
    pub rank: u32
}

impl ImgCompressor {
    pub fn new(img_res: ImageResult<DynamicImage>) -> ImgCompressor {
        let rgb = img_res.unwrap().to_rgb();
        let pixel_buffer = rgb.pixels();
        let mut data = vec![(0u8,0u8,0u8,0u8); pixel_buffer.len()];
        for (i, pixel) in pixel_buffer.enumerate() {
            data[i] = pixel.channels4();
        }
        let rank = (data.len() as f32).sqrt() as u32;
        assert!(data.len() as u32 == rank * rank);
        let img = ImgData { pixels: &data, rank };
        let quadtree = Quadtree::build(&img, (0, 0), rank);
        let root = Box::new(Quadtree::Root(rank as u32, quadtree));
        return ImgCompressor { root, rank };
    }
    pub fn build_tree(data: &Vec<Pix>) -> Box<Quadtree> {
        let rank = (data.len() as f32).sqrt() as u32;
        assert!(data.len() as u32 == rank * rank);
        let img = ImgData { pixels: data, rank };
        let quadtree = Quadtree::build(&img, (0, 0), rank);
        return Box::new(Quadtree::Root(rank as u32, quadtree));
    }
    pub fn leaf_index(&self, compression: (u8, u8, u8)) -> BitVec {
        let node_count = self.root.size(compression);
        let mut quad_index = BitVec::from_elem(node_count, true);
        self.root.build_index(&mut quad_index);
        return quad_index;
    }
    pub fn leaf_data(&self) -> Vec<Pix> {
        let mut leaf_data = vec![(0u8,0u8,0u8,0u8); 0];
        self.root.build_leaf_data(&mut leaf_data);
        return leaf_data;
    }
    pub fn to_image(&self, cutoffs: (u8, u8, u8)) -> RgbImage {
        let rank = self.rank;
        return ImageBuffer::from_fn(rank, rank, |x, y| {
            let r = PixelReq{ x, y, chan: 0, cutoff: cutoffs.0 };
            let g = PixelReq{ x, y, chan: 1, cutoff: cutoffs.1 };
            let b = PixelReq{ x, y, chan: 2, cutoff: cutoffs.2 };
            image::Rgb([
                self.root.get(r, 0, 0, rank),
                self.root.get(g, 0, 0, rank),
                self.root.get(b, 0, 0, rank),
            ])
        });
    }
}