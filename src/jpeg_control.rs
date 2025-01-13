//========================================================
//  jpeg_control.rs
//
//========================================================
use crate::jpeg_raw_data;
use crate::jpeg_frame_info;
use crate::jpeg_sample_block;
use crate::jpeg_huffman_table;

#[allow(dead_code)]
enum JpegMarker
{
    SOF0 = 0xFFC0,
    DHT  = 0xFFC4,
    SOI  = 0xFFD8,
    EOI  = 0xFFD9,
    SOS  = 0xFFDA,
    DQT  = 0xFFDB,
    APP0 = 0xFFE0,
    APP1 = 0xFFE1,
}

pub struct JpegControl
{
    rawdata: jpeg_raw_data::JpegRawData,
    frame_header_info: jpeg_frame_info::JpegFrameHeaderInfo,
    dht_mgr: jpeg_huffman_table::JpegDhtManager,
    img_start: usize,
}

#[allow(dead_code)]
impl JpegControl
{
    // Constructor
    pub fn new() -> Self
    {
        JpegControl
        {
            rawdata: jpeg_raw_data::JpegRawData::new(),
            frame_header_info: jpeg_frame_info::JpegFrameHeaderInfo::new(),
            dht_mgr: jpeg_huffman_table::JpegDhtManager::new(),
            img_start: 0,
        }
    }

    // ファイル読み込み
    pub fn read_from_file(&mut self, infilename: &String)
    {
        self.rawdata.read_from_file(infilename);
    }

    // JPEG マーカーの parse
    pub fn parse_markers(&mut self)
    {
        let mut reader = jpeg_raw_data::JpegReader::new(&mut self.rawdata);

        while !reader.is_end()
        {
            let seg_size: usize;
            let marker_name: &'static str;
            let m = reader.read_u16be();
            if m == JpegMarker::SOI as u16
            {
                seg_size = 0;
                marker_name = "SOI ";
            }
            else if m == JpegMarker::EOI as u16
            {
                seg_size = 0;
                marker_name = "EOI ";
            }
            else
            {
                seg_size = reader.read_u16be() as usize;
                let mut reader2 = reader.copy();
                if m == JpegMarker::DHT as u16
                {
                    marker_name = "DHT ";
                    self.dht_mgr.parse_segment(&mut reader2);
                }
                else if m == JpegMarker::DQT as u16
                {
                    marker_name = "DQT ";
                }
                else if m == JpegMarker::SOF0 as u16
                {
                    marker_name = "SOF0";
                    self.frame_header_info.parse_segment(&mut reader2);
                }
                else if m == JpegMarker::APP0 as u16
                {
                    marker_name = "APP0";
                }
                else if m == JpegMarker::APP1 as u16
                {
                    marker_name = "APP1";
                }
                else if m == JpegMarker::SOS as u16
                {
                    marker_name = "SOS ";
                    self.img_start = reader.get_pos() + seg_size - 2;
                }
                else
                {
                    marker_name = "....";
                }
                reader.move_pos(seg_size as isize - 2);
            }
            print!("{} {:04x} \n", marker_name, seg_size);
        }
        
        self.frame_header_info.dump();
        //self.dht_mgr.dump();

        let mut bsreader = jpeg_raw_data::JpegBitStreamReader::new(&mut self.rawdata);
        let mut mcu = jpeg_sample_block::JpegMinimumCodedUnit::new();
        mcu.set_mode(&self.frame_header_info);

        bsreader.set_pos(self.img_start, 0);
        self.dht_mgr.decode(&mut bsreader, &mut mcu);
    }
}

//========================================================
