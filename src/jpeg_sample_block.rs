//========================================================
//  jpeg_sample_block.rs
//
//========================================================
use crate::jpeg_frame_info;
use crate::jpeg_frame_info::JPEG_MAX_NUM_OF_COMPONENTS;

const JPEG_SAMPLE_BLOCK_SIZE: usize = 64;
const JPEG_MCU_MAX_NUM_BLOCKS: usize = 6;

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

    fn add_coefficients(&mut self, coef: i16, num_zero_run: usize) -> bool
    {
        self.sample[self.index] = coef;
        self.index += 1;

        let mut count_zero = num_zero_run;
        while self.index < JPEG_SAMPLE_BLOCK_SIZE && count_zero > 0
        {
            self.sample[self.index] = coef;
            self.index += 1;
            count_zero -= 1;   
        }
        self.index == JPEG_SAMPLE_BLOCK_SIZE
    }

    fn dump(&self)
    {
        for i in 0..64
        {
            print!("{} ", self.sample[i]);
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

    pub fn get_current_dht_id(&self) -> usize
    {
        self.dht_ids[self.index]
    }

    pub fn is_completed(&self) -> bool
    {
        self.index >= self.num_blocks_in_mcu
    }

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
