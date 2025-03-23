//========================================================
//  jpeg_sample_block.rs
//
//========================================================
use crate::jpeg_constants::
{
    JPEG_SAMPLE_BLOCK_SIZE,
    JPEG_MAX_NUM_OF_COMPONENTS,
    JPEG_REV_ZIGZAG_TABLE,
};
use crate::jpeg_frame_info;
use crate::jpeg_raw_data::JpegBitStreamReader;
use crate::jpeg_huffman_table::JpegDhtManager;
use crate::jpeg_quantization_table::JpegDqtManager;
use crate::jpeg_idct::JpegIdctManager;
use crate::jpeg_sampler::JpegSampler;

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
pub struct JpegSampleBlock
{
    sample: [i16; JPEG_SAMPLE_BLOCK_SIZE],
    index: usize,
}

pub struct JpegMinimumCodedUnit
{
    blocks: [JpegSampleBlock; JPEG_MCU_MAX_NUM_BLOCKS],
    component_ids: [u8; JPEG_MCU_MAX_NUM_BLOCKS],
    dht_ids: [u8; JPEG_MAX_NUM_OF_COMPONENTS],
    sampling_factor: [jpeg_frame_info::JpegSamplingFactor; JPEG_MAX_NUM_OF_COMPONENTS],
    last_dc: [i16; JPEG_MAX_NUM_OF_COMPONENTS],
    sampler: JpegSampler,
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

    fn reset_index(&mut self)
    {
        self.index = 0;
    }

    // Zigzag order index
    fn get_zigzag_index(&mut self) -> usize
    {
        let zi = JPEG_REV_ZIGZAG_TABLE[self.index] as usize;
        self.index += 1;
        zi
    }

    // Add coefficients from huffman-decoded stream
    fn add_coefficients(&mut self, coef: i16, num_zero_run: usize) -> bool
    {
        let mut count_zero = num_zero_run;
        while self.index < JPEG_SAMPLE_BLOCK_SIZE - 1 && count_zero > 0
        {
            self.sample[self.get_zigzag_index()] = 0;
            count_zero -= 1;   
        }
        self.sample[self.get_zigzag_index()] = coef;

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

    fn transform(&mut self)
    {
        let mut tm = JpegIdctManager::new();
        tm.idct(&mut self.sample);
    }

    pub fn iter(&self) -> std::slice::Iter<i16>
    {
        self.sample.iter()
    }

    fn dump(&self)
    {
        for i in 0..JPEG_SAMPLE_BLOCK_SIZE
        {
            print!("{:4} ", self.sample[i]);
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
            component_ids: [0; JPEG_MCU_MAX_NUM_BLOCKS],
            dht_ids: [0; JPEG_MAX_NUM_OF_COMPONENTS],
            sampling_factor: [jpeg_frame_info::JpegSamplingFactor::new(); JPEG_MAX_NUM_OF_COMPONENTS],
            last_dc: [0; JPEG_MAX_NUM_OF_COMPONENTS],
            sampler: JpegSampler::new(),
            index: 0,
            num_blocks_in_mcu: JPEG_MCU_MAX_NUM_BLOCKS,
        }
    }

    fn reset(&mut self)
    {
        self.index = 0;
        for i in 0..JPEG_MCU_MAX_NUM_BLOCKS
        {
            self.blocks[i].reset_index();
        }
    }

    fn get_current_component_id(&self) -> usize
    {
        self.component_ids[self.index] as usize
    }

    fn get_current_table_id(&self) -> usize
    {
        self.dht_ids[self.get_current_component_id()] as usize
    }

    // Add coefficients from huffman-decoded stream
    fn add_coefficients(&mut self, coef: i16, num_zero_run: usize) -> bool
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

    // Add coefficients from huffman-decoded stream (for DC coefficient)
    fn add_coefficients_dc(&mut self, coef: i16) -> bool
    {
        // DC value is an offset from the last one.
        let cid = self.get_current_component_id();
        let coef1 = coef + self.last_dc[cid];
        self.last_dc[cid] = coef1;
        self.add_coefficients(coef1, 0)
    }

    // Fill coeffieients through an Huffman-encoded bitstream
    pub fn fill_coefficients(&mut self, dht: &JpegDhtManager, bsreader: &mut JpegBitStreamReader)
    {
        self.reset();

        while self.index < self.num_blocks_in_mcu
        {
            let table_id = self.get_current_table_id();
            let dc_decoded = dht.decode_dc(table_id, bsreader);
            self.add_coefficients_dc(dc_decoded);
            let mut is_end = false;
            while !is_end
            {
                let ac_decoded = dht.decode_ac(table_id, bsreader);
                is_end = self.add_coefficients(ac_decoded.0, ac_decoded.1);
            }
        }
    }

    // Scale coefficients for dequantization
    pub fn dequantize(&mut self, dqt: &JpegDqtManager)
    {
        self.reset();

        while self.index < self.num_blocks_in_mcu
        {
            let table_id = self.get_current_table_id();
            self.blocks[self.index].scale_coefficients(dqt.get_qt_slice(table_id));
            self.index += 1;
        }
    }

    // (Inverse) discrete-cosine transform
    pub fn transform(&mut self)
    {
        for i in 0..self.num_blocks_in_mcu
        {
            self.blocks[i].transform();
        }
    }

    // Up-sampling
    pub fn upsampling(&self, out_buf: &mut [u8])
    {
        self.sampler.upsampling(&self.blocks, out_buf);
    }

    // Sets MCU mode via component sampling information
    pub fn set_mode(&mut self, fh: &jpeg_frame_info::JpegFrameHeaderInfo)
    {
        let mut i: usize = 0; 
        for j in 0..fh.get_num_components()
        {
            self.sampling_factor[j] = fh.get_sampling_factor(j);
            self.dht_ids[j] = fh.get_table_id(j) as u8;
            let num_blocks = self.sampling_factor[j].get_num_blocks();
            for _k in 0..num_blocks
            {
                self.component_ids[i] = j as u8;
                i += 1;
            }
        }
        self.num_blocks_in_mcu = i;
    }

    pub fn dump(&self)
    {
        for i in 0..self.num_blocks_in_mcu
        {
            let cid = self.component_ids[i] as usize;
            println!("Block {} (ComponentID={}, TableID={}):", i, cid, self.dht_ids[cid]);
            self.blocks[i].dump();
        }
    }
}

//========================================================
