//========================================================
//  jpeg_frame_info.rs
//
//========================================================
use crate::jpeg_constants::JPEG_MAX_NUM_OF_COMPONENTS;
use crate::jpeg_raw_data;

const JPEG_MCU_NUM_PIXELS_MIN: usize = 8;

#[derive(Copy)]
#[derive(Clone)]
pub struct JpegSamplingFactor
{
    val: u8,
}

pub struct JpegFrameHeaderInfo
{
    height: u16,
    width: u16,
    precision: u8,
    num_components: u8,
    sampling_factor: [JpegSamplingFactor; JPEG_MAX_NUM_OF_COMPONENTS],
    qt_selector: [u8; JPEG_MAX_NUM_OF_COMPONENTS],
}

#[allow(dead_code)]
impl JpegSamplingFactor
{
    // Constructor
    pub fn new() -> Self
    {
        JpegSamplingFactor
        {
            val: 0,
        }
    }

    pub fn set(&mut self, val: u8)
    {
        self.val = val;
    }

    pub fn get_raw(&self) -> usize
    {
        self.val as usize
    }

    pub fn get_num_h(&self) -> usize
    {
        (self.val >> 4) as usize
    }

    pub fn get_num_v(&self) -> usize
    {
        (self.val & 0x0F) as usize
    }

    pub fn get_num_blocks(&self) -> usize
    {
        self.get_num_v() * self.get_num_h()
    }

    pub fn get_num_mcu_pixels_h(&self) -> usize
    {
        self.get_num_h() * JPEG_MCU_NUM_PIXELS_MIN
    }

    pub fn get_num_mcu_pixels_v(&self) -> usize
    {
        self.get_num_v() * JPEG_MCU_NUM_PIXELS_MIN
    }
}

#[allow(dead_code)]
impl JpegFrameHeaderInfo
{
    // Constructor
    pub fn new() -> Self
    {
        JpegFrameHeaderInfo
        {
            height: 0,
            width: 0,
            precision: 0,
            num_components: 0,
            sampling_factor: [JpegSamplingFactor::new(); JPEG_MAX_NUM_OF_COMPONENTS],
            qt_selector: [0; JPEG_MAX_NUM_OF_COMPONENTS],
        }
    }

    // セグメント内容の parse と読み込み
    pub fn parse_segment(&mut self, reader: &mut jpeg_raw_data::JpegReader)
    {
        // Precision
        self.precision = reader.read_u8();

        // Height / Width
        self.height = reader.read_u16be();
        self.width = reader.read_u16be();

        // Number of components
        self.num_components = reader.read_u8();

        // Component info
        for _i in 0..self.num_components as usize
        {
            let component_id = reader.read_u8() as usize - 1;
            self.sampling_factor[component_id].set(reader.read_u8());
            self.qt_selector[component_id] = reader.read_u8();
        }
    }

    // Number of components
    pub fn get_num_components(&self) -> usize
    {
        self.num_components as usize
    }

    // Precision
    pub fn get_precision(&self) -> usize
    {
        self.precision as usize
    }

    // Width/Height
    pub fn get_dimension(&self) -> (usize, usize)
    {
        (self.width as usize, self.height as usize)
    }

    // Sampling factor
    pub fn get_sampling_factor(&self, index: usize) -> JpegSamplingFactor
    {
        assert!(index < self.num_components as usize);
        self.sampling_factor[index]
    }

    // Quantization table selector
    pub fn get_table_id(&self, index: usize) -> usize
    {
        assert!(index < self.num_components as usize);
        self.qt_selector[index] as usize
    }

    // 構造体内容のダンプ
    pub fn dump(&self)
    {
        println!("\n---- Frame Header Info. ----");
        // Precision, Num components
        println!("Precision= {} / Num components= {}", self.precision, self.num_components);
        // Width / Height
        println!("Width= {} / Height= {}", self.width, self.height);

        // Component descriptors
        for i in 0..self.num_components as usize
        {
            println!(
                "C={} / HV={},{} / TQ={}",
                i+1,
                self.sampling_factor[i].get_num_h(),
                self.sampling_factor[i].get_num_v(),
                self.qt_selector[i]
            );
        }
        println!("----------------");
    }
}

//========================================================
