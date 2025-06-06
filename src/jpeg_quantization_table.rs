//========================================================
//  jpeg_quantization_table.rs
//
//========================================================
use crate::jpeg_constants::JPEG_SAMPLE_BLOCK_SIZE;
use crate::jpeg_constants::JPEG_REV_ZIGZAG_TABLE;
use crate::jpeg_raw_data::JpegReader;

const JPEG_NUM_DQT: usize = 4;

#[derive(Copy)]
#[derive(Clone)]
struct JpegQuantizationTable
{
    sample: [u16; JPEG_SAMPLE_BLOCK_SIZE],
}

pub struct JpegDqtManager
{
    qt: [JpegQuantizationTable; JPEG_NUM_DQT],
}

#[allow(dead_code)]
impl JpegQuantizationTable
{
    // Constructor
    fn new() -> Self
    {
        JpegQuantizationTable
        {
            sample: [0; JPEG_SAMPLE_BLOCK_SIZE],
        }
    }

    // セグメント内容の parse と読み込み
    fn read_table(&mut self, reader: &mut JpegReader)
    {
        // Quantization table info
        for i in 0..JPEG_SAMPLE_BLOCK_SIZE
        {
            self.sample[JPEG_REV_ZIGZAG_TABLE[i] as usize] = reader.read_u8() as u16;
        }       
    }

    fn get_slice(&self) -> &[u16]
    {
        &self.sample
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
    }
}

#[allow(dead_code)]
impl JpegDqtManager
{
    // Constructor
    pub fn new() -> Self
    {
        JpegDqtManager
        {
            qt: [JpegQuantizationTable::new(); JPEG_NUM_DQT],
        }
    }

    // セグメント内容の parse と読み込み
    pub fn read_table(&mut self, reader: &mut JpegReader)
    {
        // どのテーブルを使用するかのため ID を先読み
        let id = reader.read_u8();

        // TODO: 16-bit 精度(precision=1)未対応
        let precision = id >> 4;
        assert!(precision == 0);
 
        let idx = (id & 0x03) as usize;
        self.qt[idx].read_table(reader);
    }

    // Get quantization table as a slice
    pub fn get_qt_slice(&self, table_id: usize) -> &[u16]
    {
        self.qt[table_id].get_slice()
    }

    // 全 DQT テーブルのダンプ
    pub fn dump(&self)
    {
        for i in 0..JPEG_NUM_DQT
        {
            println!("[DQT {}]", i);
            self.qt[i].dump();
        }
    }
}

//========================================================
