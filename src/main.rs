mod compressor;
mod quadtree;
use std::{env, fs};

use crate::compressor::ImgCompressor;

fn main() {
    let compression = parse_arguments(env::args().collect());
    let compressor = ImgCompressor::new(image::open("./samples/lena.png"));
    let outfile = "./samples/output.png";
    let res = compressor.to_image(compression).save(outfile);
    let size = compressor.compressed_size(compression);
    let data = compressor.to_file(compression);
    let _ = fs::write("./samples/output.ski", data);
    println!("success: {}, compressed byte size: {} B", res.is_ok(), size);
}

fn parse_arguments(args: Vec<String>) -> (u8, u8, u8) {
    if args.len() == 4 {
        return (
            args.get(1).unwrap().parse::<u8>().unwrap(),
            args.get(2).unwrap().parse::<u8>().unwrap(),
            args.get(3).unwrap().parse::<u8>().unwrap(),
        )
    }
    return (2, 2, 2);
}
