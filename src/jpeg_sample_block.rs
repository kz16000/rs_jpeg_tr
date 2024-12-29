//========================================================
//  jpeg_sample_block.rs
//
//========================================================

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
            index: 0,
            num_blocks_in_mcu: JPEG_MCU_MAX_NUM_BLOCKS,
        }
    }

    fn set_mode_internal(&mut self, num_y_blocks: usize, num_c_blocks: usize)
    {
        self.num_blocks_in_mcu = num_y_blocks + num_c_blocks;
        for i in 0..self.num_blocks_in_mcu
        {
            self.dht_ids[i] = if i >= num_y_blocks
            {
                1
            }
            else
            {
                0
            }
        }
    }

    pub fn set_mode(&mut self, mode: JpegSampleMode)
    {
        match mode
        {
            JpegSampleMode::JpegModeYuv444 => self.set_mode_internal(1, 2),
            JpegSampleMode::JpegModeYuv422 => self.set_mode_internal(2, 2),
            JpegSampleMode::JpegModeYuv420 => self.set_mode_internal(4, 2),
        }
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
            println!("Block {}:", i);
            self.blocks[i].dump();
        }
    }
}

//========================================================
