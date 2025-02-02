//========================================================
//  jpeg_huffman_table.rs
//
//========================================================
use crate::jpeg_raw_data::JpegReader;
use crate::jpeg_raw_data::JpegBitStreamReader;
use crate::jpeg_sample_block::JpegMinimumCodedUnit;
use crate::jpeg_frame_info::JPEG_MAX_NUM_OF_COMPONENTS;

const JPEG_NUM_DHT_TREE_BITS: usize = 16;
const JPEG_DHT_LOG_DETAIL: u8 = 0x01;

struct JpegHuffmanTable
{
    tree: [u8; JPEG_NUM_DHT_TREE_BITS],
    encoding: Vec<u8>,
    bit_pattern: Vec<u16>,
    bit_length: Vec<u8>,
    table_id: u8,
    log_control: u8,
}

#[allow(dead_code)]
pub struct JpegDhtManager
{
    dc: [JpegHuffmanTable; 2],
    ac: [JpegHuffmanTable; 2],
    previous_dc: [i16; JPEG_MAX_NUM_OF_COMPONENTS],
}

#[allow(dead_code)]
impl JpegHuffmanTable
{
    // Constructor
    fn new() -> Self
    {
        JpegHuffmanTable
        {
            tree: [0; JPEG_NUM_DHT_TREE_BITS],
            encoding: Vec::new(),
            bit_pattern: Vec::new(),
            bit_length:Vec::new(),
            table_id: 0,
            log_control: 0,
        }
    }

    // セグメント内容の parse と読み込み
    fn parse_segment(&mut self, reader: &mut JpegReader)
    {
        // Segment size
        let seg_size = reader.read_u16be();

        // Table ID, AD/DC flag
        self.table_id = reader.read_u8();

        // Huffman tree info
        for i in 0..JPEG_NUM_DHT_TREE_BITS
        {
            self.tree[i] = reader.read_u8();
        }

        // Encoding info
        let encoding_size = seg_size as usize - (JPEG_NUM_DHT_TREE_BITS + 3);
        self.encoding = Vec::<u8>::with_capacity(encoding_size);
        for _i in 0..encoding_size
        {
            let e = reader.read_u8();
            self.encoding.push(e);
        }

        // Bit pattern
        self.bit_pattern = Vec::<u16>::with_capacity(encoding_size);
        self.bit_length = Vec::<u8>::with_capacity(encoding_size);
        self.create_bit_pattern();
    }

    // Log control
    fn is_log_enabled(&self, flag: u8) -> bool
    {
        self.log_control & flag != 0
    }

    // Set log control
    fn set_log_control(&mut self, flag: u8)
    {
        self.log_control = flag;
    }

    // 参照用ビットパターンの展開
    fn create_bit_pattern(&mut self)
    {
        let mut pat: u16 = 0;
        let mut base: u16 = 0x8000;
        for i in 0..JPEG_NUM_DHT_TREE_BITS
        {
            for _j in 0..self.tree[i]
            {
                self.bit_pattern.push(pat);
                self.bit_length.push((i+1) as u8);
                pat += base;
            }
            base >>= 1;
        }
    }

    // SSSS ビット数に応じた係数値の展開
    fn unpack_coefficient(&self, n_bits_ssss: u8, packed_data: u16) -> i16
    {
        let bs = packed_data >> (16 - n_bits_ssss);
        if (packed_data & 0x8000) == 0
        {
            // MSB=0 -> negative
            - (1 << n_bits_ssss) + 1 + bs as i16
        }
        else
        {
            // MSB=1 -> positive
            bs as i16
        }       
    }

    // ビット列 Decode (DC)
    fn decode_dc(&self, bsreader: &mut JpegBitStreamReader) -> i16
    {
        let bh = bsreader.read_bits16();
        let mut n_bits_huff: u8 = 0;
        let mut n_bits_ssss: u8 = 0;
        for i in 0..self.bit_pattern.len()
        {
            if self.bit_pattern[i] > bh
            {
                n_bits_ssss = self.encoding[i - 1];
                n_bits_huff = self.bit_length[i - 1];
                if self.is_log_enabled(JPEG_DHT_LOG_DETAIL)
                {
                    println!("DC: Match {} @ {:016b} {:016b} {} -> {}",
                            i, bh, self.bit_pattern[i-1], n_bits_huff, n_bits_ssss);
                }
                break;
            }
        }
        bsreader.move_bitpos(n_bits_huff as isize);
        let mut bs: u16 = 0;
        let mut dc_diff: i16 = 0;
        // DC coding table conversion
        if n_bits_ssss > 0
        {
            bs = bsreader.read_bits16();
            bsreader.move_bitpos(n_bits_ssss as isize);
            dc_diff = self.unpack_coefficient(n_bits_ssss, bs);
        }
        if self.is_log_enabled(JPEG_DHT_LOG_DETAIL)
        {
            println!("SSSS Unpacked data: {:016b} @ {} -> {}", bs, n_bits_ssss, dc_diff);
        }
        dc_diff
    }

    // ビット列 Decode (AC)
    fn decode_ac(&self, bsreader: &mut JpegBitStreamReader) -> (i16, usize)
    {
        let bh = bsreader.read_bits16();
        let mut n_bits_huff: u8 = 0;
        let mut n_bits_ssss: u8 = 0;
        let mut n_zero_run: u8 = 0;
        for i in 0..self.bit_pattern.len()
        {
            if self.bit_pattern[i] > bh
            {
                n_bits_ssss = self.encoding[i - 1];               
                n_zero_run = if n_bits_ssss != 0
                {
                    n_bits_ssss >> 4    // upper 4bit
                }
                else
                {
                    255 // special case (fill remaining blocks with zero)
                };
                n_bits_ssss = n_bits_ssss & 0xF;    // lower 4bit
                n_bits_huff = self.bit_length[i - 1];
                if self.is_log_enabled(JPEG_DHT_LOG_DETAIL)
                {
                    println!("AC: Match {} @ {:016b} {:016b} {} -> {}",
                            i, bh, self.bit_pattern[i-1], n_bits_huff, n_bits_ssss);
                }
                break;
            }
        }
        bsreader.move_bitpos(n_bits_huff as isize);
        let mut bs: u16 = 0;
        let mut dc_diff: i16 = 0;
        // AC coding table conversion
        if n_bits_ssss > 0
        {
            bs = bsreader.read_bits16();
            bsreader.move_bitpos(n_bits_ssss as isize);
            dc_diff = self.unpack_coefficient(n_bits_ssss, bs);
        }
        if self.is_log_enabled(JPEG_DHT_LOG_DETAIL)
        {
            println!("Zero run-length: {}", n_zero_run);
            println!("SSSS Unpacked data: {:016b} @ {} -> {}", bs, n_bits_ssss, dc_diff);
        }
        (dc_diff, n_zero_run as usize)
    }

    // 構造体内容のダンプ
    fn dump(&self)
    {
        // Table ID, AD/DC flag
        println!("{:02x}", self.table_id);

        // Huffman tree info
        for i in 0..JPEG_NUM_DHT_TREE_BITS
        {
            print!("{:02x} ", self.tree[i]);
        }
        println!("\n--------");

        // Encoding info
        for i in 0..self.encoding.len()
        {
            print!("{:02x} ", self.encoding[i]);
        }
        println!("\n--------");

        // Bit pattern
        for i in 0..self.bit_pattern.len()
        {
            print!("{:04x} [{:02}] ", self.bit_pattern[i], self.bit_length[i]);
        }
        println!("\n--------");
    }
}

#[allow(dead_code)]
impl JpegDhtManager
{
    // Constructor
    pub fn new() -> Self
    {
        JpegDhtManager
        {
            ac: [JpegHuffmanTable::new(), JpegHuffmanTable::new()],
            dc: [JpegHuffmanTable::new(), JpegHuffmanTable::new()],
            previous_dc: [0; JPEG_MAX_NUM_OF_COMPONENTS],
        }
    }

    // セグメント内容の parse と読み込み
    pub fn parse_segment(&mut self, reader: &mut JpegReader)
    {
        // どのテーブルを使用するかのため ID を先読み
        let id = reader.read_u8();
        // 一旦セクションの先頭に巻き戻し
        reader.move_pos(-3);
 
        let idx = (id & 0x03) as usize;
        if id & 0x10 == 0
        {
           self.dc[idx].parse_segment(reader);
        }
        else
        {
           self.ac[idx].parse_segment(reader);
        }       
    }
    
    // Decode
    pub fn decode(&self, bsreader: &mut JpegBitStreamReader, mcu: &mut JpegMinimumCodedUnit)
    {
        mcu.reset();

        while !mcu.is_completed()
        {
            let table_id = mcu.get_current_table_id();
            let dc_decoded = self.dc[table_id].decode_dc(bsreader);
            mcu.add_coefficients_dc(dc_decoded);
            let mut is_end = false;
            while !is_end
            {
                let ac_decoded = self.ac[table_id].decode_ac(bsreader);
                is_end = mcu.add_coefficients(ac_decoded.0, ac_decoded.1);
            }
        }
    }

    // Set log control
    fn set_log_control(&mut self, flag: u8)
    {
        self.dc[0].set_log_control(flag);
        self.dc[1].set_log_control(flag);
        self.ac[0].set_log_control(flag);
        self.ac[1].set_log_control(flag);
    }

    // 読み込み済の全 DHT テーブルのダンプ
    pub fn dump(&self)
    {
        println!("[DHT DC Table 0]");
        self.dc[0].dump();
        println!("[DHT DC Table 1]");
        self.dc[1].dump();
        println!("[DHT AC Table 0]");
        self.ac[0].dump();
        println!("[DHT AC Table 1]");
        self.ac[1].dump();
    }
}

//========================================================
