mod compressor;
mod quadtree;

use crate::compressor::ImgCompressor;


fn main() {
    let compression = (50, 4, 100);
    let compressor = ImgCompressor::new(image::open("./samples/lena.png"));
    let res = compressor.to_image(compression).save("./samples/output.png");
    println!("saved image: {}", res.is_ok());
}
