mod compressor;
mod quadtree;
use std::fs;

use crate::compressor::ImgCompressor;

fn main() {
    let compression = (8, 30, 30);
    let compressor = ImgCompressor::new(image::open("./samples/lena.png"));
    let cfg = compression;
    let outfile = format!("./samples/output.png");
    let res = compressor.to_image(compression).save(outfile);
    let size = compressor.compressed_size(compression);
    let data = compressor.to_file(compression);    
    fs::write("./samples/output.ski", data);
    println!("success: {}, compressed byte size: {} B", res.is_ok(), size);
}
