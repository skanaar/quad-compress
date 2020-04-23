use bit_vec::BitVec;
use image::{ RgbImage, DynamicImage, ImageBuffer, Pixel };
use image::error::ImageResult;

use crate::quadtree::{ Quadtree, Pix, ImgData, Cutoff, ChannelReq };

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
        let root = Quadtree::build(&img, (0, 0), rank);
        return ImgCompressor { root, rank };
    }
    pub fn leaf_index(&self, request: ChannelReq) -> BitVec {
        let mut quad_index = BitVec::new();
        self.root.build_leaf_index(&mut quad_index, request);
        return quad_index;
    }
    pub fn leaf_data(&self, request: ChannelReq) -> Vec<u8> {
        let mut leaf_data = vec![0u8; 0];
        self.root.build_leaf_data(&mut leaf_data, request);
        return leaf_data;
    }
    pub fn compressed_size(&self, cutoffs: Cutoff) -> usize {
        let rd = self.leaf_data(ChannelReq{ chan: 0, cutoff: cutoffs.0 });
        let gd = self.leaf_data(ChannelReq{ chan: 1, cutoff: cutoffs.1 });
        let bd = self.leaf_data(ChannelReq{ chan: 2, cutoff: cutoffs.2 });
        let ri = self.leaf_index(ChannelReq{ chan: 0, cutoff: cutoffs.0 });
        let gi = self.leaf_index(ChannelReq{ chan: 1, cutoff: cutoffs.1 });
        let bi = self.leaf_index(ChannelReq{ chan: 2, cutoff: cutoffs.2 });
        return rd.len()+gd.len()+bd.len() + (ri.len()+gi.len()+bi.len())/8;
    }
    pub fn to_image(&self, cutoffs: Cutoff) -> RgbImage {
        let rank = self.rank;
        return ImageBuffer::from_fn(rank, rank, |x, y| {
            let r = ChannelReq{ chan: 0, cutoff: cutoffs.0 };
            let g = ChannelReq{ chan: 1, cutoff: cutoffs.1 };
            let b = ChannelReq{ chan: 2, cutoff: cutoffs.2 };
            image::Rgb([
                self.root.get((x, y), r, (0, 0), rank),
                self.root.get((x, y), g, (0, 0), rank),
                self.root.get((x, y), b, (0, 0), rank),
            ])
        });
    }
}