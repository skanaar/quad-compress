use bitvec::prelude::Local;
use bitvec::vec::BitVec;
use image::{ RgbImage, DynamicImage, ImageBuffer, Pixel };
use image::error::ImageResult;
use crate::quadtree::Quadtree;
use crate::serialize::{ build_leaf_index, build_leaf_data };

type Pix = (u8, u8, u8, u8);
pub type Cutoff = (u8, u8, u8);

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

fn ycca_to_rgba(ycc: Pix) -> Pix {
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
    pub lumin_root: Box<Quadtree>,
    pub c_blu_root: Box<Quadtree>,
    pub c_red_root: Box<Quadtree>,
    pub rank: u32
}

impl ImgCompressor {
    pub fn new(img_res: ImageResult<DynamicImage>) -> ImgCompressor {
        let rgb = img_res.unwrap().to_rgb8();
        let pixel_buffer = rgb.pixels();
        let pixel_len = pixel_buffer.len();
        let mut lumin = vec![0u8; pixel_buffer.len()];
        let mut c_blu = vec![0u8; pixel_buffer.len()];
        let mut c_red = vec![0u8; pixel_buffer.len()];
        for (i, pixel) in pixel_buffer.enumerate() {
            let ycca = rgb_to_ycc(pixel.channels4());
            lumin[i] = ycca.0;
            c_blu[i] = ycca.1;
            c_red[i] = ycca.2;
        }
        let rank = (pixel_len as f32).sqrt() as u32;
        assert!(pixel_len as u32 == rank * rank);
        let lumin_root = Quadtree::new(&lumin);
        let c_blu_root = Quadtree::new(&c_blu);
        let c_red_root = Quadtree::new(&c_red);
        return ImgCompressor { lumin_root, c_blu_root, c_red_root, rank };
    }

    fn predicted_capacity(&self) -> usize {
        return (self.rank * self.rank) as usize;
    }

    fn leaf_index(&self, quadtree_root: &Box<Quadtree>, cutoff: u8) -> BitVec<Local, u8> {
        let mut quad_index: BitVec<Local, u8> = BitVec::with_capacity(self.predicted_capacity());
        build_leaf_index(quadtree_root, &mut quad_index, cutoff);
        return quad_index;
    }

    fn leaf_data(&self, quadtree_root: &Box<Quadtree>, cutoff: u8) -> Vec<u8> {
        let mut leaf_data = Vec::with_capacity(self.predicted_capacity());
        build_leaf_data(quadtree_root, &mut leaf_data, cutoff);
        return leaf_data;
    }

    pub fn to_file(&self, cutoffs: Cutoff) -> Vec<u8> {
        let r_index = self.leaf_index(&self.lumin_root, cutoffs.0).into_vec();
        let g_index = self.leaf_index(&self.c_blu_root, cutoffs.1).into_vec();
        let b_index = self.leaf_index(&self.c_red_root, cutoffs.2).into_vec();
        let r_leaf = self.leaf_data(&self.lumin_root, cutoffs.0);
        let g_leaf = self.leaf_data(&self.c_blu_root, cutoffs.1);
        let b_leaf = self.leaf_data(&self.c_red_root, cutoffs.2);
        let bytes = [
            &r_index[..],
            &g_index[..],
            &b_index[..],
            &r_leaf[..],
            &g_leaf[..],
            &b_leaf[..]
        ].concat();
        return bytes;
    }

    pub fn to_image(&self, cutoffs: Cutoff) -> RgbImage {
        let rank = self.rank;
        let img = ImageBuffer::from_fn(rank, rank, |x, y| {
            let p = (x as usize, y as usize);
            let rgb = ycca_to_rgba((
                self.lumin_root.get_approx(p, cutoffs.0),
                self.c_blu_root.get_approx(p, cutoffs.1),
                self.c_red_root.get_approx(p, cutoffs.2),
                0
            ));
            image::Rgb([rgb.0,rgb.1,rgb.2])
        });
        return img;
    }
}
