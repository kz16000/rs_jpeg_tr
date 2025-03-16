//========================================================
//  main.rs
//
//========================================================
use std::env;

mod jpeg_constants;
mod jpeg_raw_data;
mod jpeg_sample_block;
mod jpeg_huffman_table;
mod jpeg_quantization_table;
mod jpeg_idct;
mod jpeg_sampler;
mod jpeg_frame_info;
mod jpeg_control;

fn main() 
{
    // コマンドライン引数読み込み
    let args: Vec<String> = env::args().collect();

    // 第1引数の取り出し
    let infilename: &String = args.get(1)
        .expect("Please give a input file name as argument.");
    println!("Filename: {}", infilename);

    // JpegFile 構造体の初期化
    let mut jpeg = jpeg_control::JpegControl::new();

    jpeg.read_from_file(infilename);
    jpeg.parse_markers();
    jpeg.decode_image();
}

//========================================================
