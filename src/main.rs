mod compressor;
mod quadtree;

use crate::compressor::ImgCompressor;

fn main() {
    let compression = (100, 50, 150);
    let compressor = ImgCompressor::new(image::open("./samples/lena.png"));
    let res = compressor.to_image(compression).save("./samples/output.png");
    let size = compressor.compressed_size(compression);
    println!("success: {}, compressed byte size: {} B", res.is_ok(), size);
}
