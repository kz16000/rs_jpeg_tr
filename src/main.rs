//========================================================
//  main.rs
//
//========================================================
use std::env;
use std::fs::File;
use std::io::Write;
use std::io::Result;

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

fn main() -> Result<()>
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

    // Image width/height
    let (width, height) = jpeg.get_dimension();

    // Allocate output buffer
    let buf_size = jpeg.get_total_buffer_size();
    let val: u8 = 0;
    let mut img_buffer = vec![val; buf_size];

    jpeg.decode_image(&mut img_buffer);

    // Dumps result image buffer as PPM ASCII format
    let mut out_file = File::create("out.ppm")?;
    let mut count = 0;
    writeln!(out_file, "P3")?;
    writeln!(out_file, "{} {}", width, height)?;
    writeln!(out_file, "255")?;
    for d in img_buffer
    {
        write!(out_file, "{} ", d)?;
        count += 1;
        if count >= 24
        {
            writeln!(out_file)?;
            count = 0;
        }
    }

    Ok(())
}

//========================================================
