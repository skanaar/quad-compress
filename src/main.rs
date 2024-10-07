mod compressor;
mod quadtree;
mod serialize;
use std::{ env, fs, os::unix::fs::MetadataExt };
use deflate::deflate_bytes;

use crate::compressor::ImgCompressor;

fn main() {
    let compression = parse_arguments(env::args().collect());
    println!(" raw     png     quad    deflat  (kB)");
    println!("------- ------- ------- -------");
    test_case(compression, "lena");
    test_case(compression, "lichtenstein");
    test_case(compression, "mandelbrot");
}

fn test_case(compression: (u8, u8, u8), name: &str) {
    let input_path = format!("./samples/{}.png", name);
    let compressor = ImgCompressor::new(image::open(&input_path));
    let outfile = format!("./output/{}.png", name);
    let png_result = compressor.to_image(compression).save(outfile);
    if !png_result.is_ok() { return; }
    let serialized_bytes = compressor.to_file(compression);
    let file_bytes = deflate_bytes(&serialized_bytes);
    let size_input = fs::metadata(&input_path).unwrap().size() / 1024;
    let size_raw = 512*512*3/1024;
    let size_a = serialized_bytes.len() / 1024;
    let size_b = file_bytes.len() / 1024;
    let ski_result = fs::write(format!("./output/{}.ski", name), file_bytes);
    if ski_result.is_ok() {
        println!("{:>4}    {:>4}    {:>4}    {:>4}    {}", size_raw, size_input, size_a, size_b, name);
    } else {
        println!("failed {}", name);
    };
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
