use bitvec::prelude::Local;
use bitvec::vec::BitVec;
use image::{ RgbImage, DynamicImage, ImageBuffer, Pixel };
use image::error::ImageResult;

use crate::quadtree::{ Quadtree, Pix, ImgData, Cutoff, ChannelReq };

fn clamp_u8(x: f32) -> u8 {
    if x < 0f32 { return 0u8 }
    else if x > 255f32 { return 255u8 }
    else { return x as u8 };
}

fn rgb_to_ycc(rgb: Pix) -> Pix {
    let r = rgb.0 as f32;
    let g = rgb.1 as f32;
    let b = rgb.2 as f32;
    return (
        clamp_u8(0.299 * r + 0.587 * g + 0.114 * b),
        clamp_u8(-0.169 * r + -0.331 * g + 0.501 * b + 128f32),
        clamp_u8(0.5 * r + -0.419 * g + -0.081 * b + 128f32),
        rgb.3,
    )
}

fn ycc_to_rgb(ycc: Pix) -> Pix {
    let y = ycc.0 as f32;
    let cb = (ycc.1 as f32) - 128.0;
    let cr = (ycc.2 as f32) - 128.0;
    return (
        clamp_u8(y + 1.402 * cr),
        clamp_u8(y - 0.344 * cb - 0.714 * cr),
        clamp_u8(y + 1.772 * cb),
        ycc.3,
    );
}

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
            data[i] = rgb_to_ycc(pixel.channels4());
        }
        let rank = (data.len() as f32).sqrt() as u32;
        assert!(data.len() as u32 == rank * rank);
        let img = ImgData { pixels: &data, rank };
        let root = Quadtree::build(&img, (0, 0), rank);
        return ImgCompressor { root, rank };
    }
    pub fn leaf_index(&self, request: ChannelReq) -> BitVec<Local, u8> {
        let mut quad_index: BitVec<Local, u8> = BitVec::new();
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
    pub fn to_file(&self, cutoffs: Cutoff) -> Vec<u8> {
        let r = ChannelReq{ chan: 0, cutoff: cutoffs.0 };
        let g = ChannelReq{ chan: 1, cutoff: cutoffs.1 };
        let b = ChannelReq{ chan: 2, cutoff: cutoffs.2 };
        let r_leaf = self.leaf_data(r);
        let g_leaf = self.leaf_data(g);
        let b_leaf = self.leaf_data(b);
        let r_index = self.leaf_index(r).into_vec();
        let g_index = self.leaf_index(g).into_vec();
        let b_index = self.leaf_index(b).into_vec();
        return [
            &r_index[..], 
            &g_index[..], 
            &b_index[..], 
            &r_leaf[..], 
            &g_leaf[..], 
            &b_leaf[..]
        ].concat();
    }
    pub fn to_image(&self, cutoffs: Cutoff) -> RgbImage {
        let rank = self.rank;
        return ImageBuffer::from_fn(rank, rank, |x, y| {
            let r = ChannelReq{ chan: 0, cutoff: cutoffs.0 };
            let g = ChannelReq{ chan: 1, cutoff: cutoffs.1 };
            let b = ChannelReq{ chan: 2, cutoff: cutoffs.2 };
            let rgb = ycc_to_rgb((
                self.root.get((x, y), r, (0, 0), rank),
                self.root.get((x, y), g, (0, 0), rank),
                self.root.get((x, y), b, (0, 0), rank),
                255,
            ));
            image::Rgb([
                rgb.0,
                rgb.1,
                rgb.2,
            ])
        });
    }
}