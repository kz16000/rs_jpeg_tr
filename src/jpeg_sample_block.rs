//========================================================
//  jpeg_sample_block.rs
//
//========================================================
use crate::jpeg_frame_info;
use crate::jpeg_frame_info::JPEG_MAX_NUM_OF_COMPONENTS;

pub const JPEG_SAMPLE_BLOCK_SIZE: usize = 64;
const JPEG_MCU_MAX_NUM_BLOCKS: usize = 6;

// Reversed zigzag table for reading coefficients
pub const REV_ZIGZAG_TABLE: [u8; JPEG_SAMPLE_BLOCK_SIZE] =
[
     0,  1,  8, 16,  9,  2,  3, 10,
    17, 24, 32, 25, 18, 11,  4,  5,
    12, 19, 26, 33, 40, 48, 41, 34,
    27, 20, 13,  6,  7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36,
    29, 22, 15, 23, 30, 37, 44, 51,
    58, 59, 52, 45, 38, 31, 39, 46,
    53, 60, 61, 54, 47, 55, 62, 63,
];

// Forward zigzag table (not used)
/*
const ZIGZAG_TABLE: [u8; JPEG_SAMPLE_BLOCK_SIZE] =
[
     0,  1,  5,  6, 14, 15, 27, 28,
     2,  4,  7, 13, 16, 26, 29, 42,
     3,  8, 12, 17, 25, 30, 41, 43,
     9, 11, 18, 24, 31, 40, 44, 53,
     10, 19, 23, 32, 39, 45, 52, 54,
     20, 22, 33, 38, 46, 51, 55, 60,
        21, 34, 37, 47, 50, 56, 59, 61,
        35, 36, 48, 49, 57, 58, 62, 63,
    ];
    */


#[allow(dead_code)]
pub enum JpegSampleMode
{
    JpegModeYuv444,
    JpegModeYuv422,
    JpegModeYuv420,
}

#[derive(Copy)]
#[derive(Clone)]
struct JpegSampleBlock
{
    sample: [i16; JPEG_SAMPLE_BLOCK_SIZE],
    index: usize,
}

pub struct JpegMinimumCodedUnit
{
    blocks: [JpegSampleBlock; JPEG_MCU_MAX_NUM_BLOCKS],
    dht_ids: [usize; JPEG_MCU_MAX_NUM_BLOCKS],
    sampling_factor: [jpeg_frame_info::JpegSamplingFactor; JPEG_MAX_NUM_OF_COMPONENTS],
    index: usize,
    num_blocks_in_mcu: usize,
}

#[allow(dead_code)]
impl JpegSampleBlock
{
    // Constructor
    fn new() -> Self
    {
        JpegSampleBlock
        {
            sample: [0; JPEG_SAMPLE_BLOCK_SIZE],
            index: 0,
        }
    }

    fn reset(&mut self)
    {
        self.index = 0;
    }

    // Zigzag order index
    fn get_zigzag_index(&mut self) -> usize
    {
        let zi = REV_ZIGZAG_TABLE[self.index] as usize;
        self.index += 1;
        zi
    }

    // Add coefficients from huffman-decoded stream
    fn add_coefficients(&mut self, coef: i16, num_zero_run: usize) -> bool
    {
        self.sample[self.get_zigzag_index()] = coef;

        let mut count_zero = num_zero_run;
        while self.index < JPEG_SAMPLE_BLOCK_SIZE && count_zero > 0
        {
            self.sample[self.get_zigzag_index()] = coef;
            count_zero -= 1;   
        }
        self.index == JPEG_SAMPLE_BLOCK_SIZE
    }

    // Scale coefficients for dequantization
    fn scale_coefficients(&mut self, scale: &[u16])
    {
        assert!(scale.len() == JPEG_SAMPLE_BLOCK_SIZE);
        for i in 0..JPEG_SAMPLE_BLOCK_SIZE
        {
            self.sample[i] = self.sample[i] * scale[i] as i16;
        }
        self.index = JPEG_SAMPLE_BLOCK_SIZE;
    }

    fn dump(&self)
    {
        for i in 0..JPEG_SAMPLE_BLOCK_SIZE
        {
            print!("{:3} ", self.sample[i]);
            if i % 8 == 7
            {
                println!();
            }
        }
        println!("index = {}", self.index);
    }
}

#[allow(dead_code)]
impl JpegMinimumCodedUnit
{
    // Constructor
    pub fn new() -> Self
    {
        JpegMinimumCodedUnit
        {
            blocks: [JpegSampleBlock::new(); JPEG_MCU_MAX_NUM_BLOCKS],
            dht_ids: [0; JPEG_MCU_MAX_NUM_BLOCKS],
            sampling_factor: [jpeg_frame_info::JpegSamplingFactor::new(); JPEG_MAX_NUM_OF_COMPONENTS],
            index: 0,
            num_blocks_in_mcu: JPEG_MCU_MAX_NUM_BLOCKS,
        }
    }

    pub fn set_mode(&mut self, fh: &jpeg_frame_info::JpegFrameHeaderInfo)
    {
        let mut i: usize = 0; 
        for j in 0..fh.get_num_components()
        {
            self.sampling_factor[j] = fh.get_sampling_factor(j);
            let num_blocks = self.sampling_factor[j].get_num_blocks();
            for _k in 0..num_blocks
            {
                self.dht_ids[i] = fh.get_table_id(j);
                i += 1;
            }
        }
        self.num_blocks_in_mcu = i;
    }

    pub fn reset(&mut self)
    {
        self.index = 0;
        for i in 0..JPEG_MCU_MAX_NUM_BLOCKS
        {
            self.blocks[i].reset();
        }
    }

    pub fn get_current_table_id(&self) -> usize
    {
        self.dht_ids[self.index]
    }

    pub fn is_completed(&self) -> bool
    {
        self.index >= self.num_blocks_in_mcu
    }

    // Add coefficients from huffman-decoded stream
    pub fn add_coefficients(&mut self, coef: i16, num_zero_run: usize) -> bool
    {
        if self.index < self.num_blocks_in_mcu
        {
            let finished = self.blocks[self.index].add_coefficients(coef, num_zero_run);
            if finished
            {
                self.index += 1;
            }
            finished
        }
        else
        {
            true
        }
    }

    // Scale coefficients for dequantization
    pub fn scale_coefficients(&mut self, scale: &[u16]) -> bool
    {
        if self.index < self.num_blocks_in_mcu
        {
            self.blocks[self.index].scale_coefficients(scale);
            self.index += 1;
        }
        self.index >= self.num_blocks_in_mcu
    }

    pub fn dump(&self)
    {
        for i in 0..self.num_blocks_in_mcu
        {
            println!("Block {} (TableID={}):", i, self.dht_ids[i]);
            self.blocks[i].dump();
        }
    }
}

//========================================================
