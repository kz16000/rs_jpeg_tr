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
mod jpeg_color_converter;
mod jpeg_sampler;
mod jpeg_frame_info;
mod jpeg_outbuffer_info;
mod jpeg_control;

fn main() 
{
    // コマンドライン引数読み込み
    let args: Vec<String> = env::args().collect();

    // Get first argument as
    let infilename: &String = args.get(1)
        .expect("Please give a input file name as argument.");
    println!("Filename: {}", infilename);

    // Initializes JpegFile structure
    let mut jpeg = jpeg_control::JpegControl::new();

    jpeg.read_from_file(infilename);
    jpeg.parse_markers();

    // Allocate output buffer
    let buf_size = jpeg.get_total_buffer_size();
    let val: u8 = 0;
    let mut img_buffer = vec![val; buf_size];

    jpeg.decode_image(&mut img_buffer);

    // Dumps result image buffer
    let mut count = 0;
    for d in img_buffer
    {
        print!("0x{:02x} ", d);
        count += 1;
        if count >= 12
        {
            println!();
            count = 0;
        }
        else if count % 3 == 0
        {
            print!("| ");
        }
    }
}

//========================================================
