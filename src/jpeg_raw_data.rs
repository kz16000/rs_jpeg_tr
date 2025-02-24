//========================================================
//  jpeg_raw_data.rs
//
//========================================================
use std::fs::File;
use std::io::prelude::*;

pub struct JpegRawData
{
    data: Vec<u8>,
    size: usize,
}

pub struct JpegReader<'a>
{
    data_ref: &'a JpegRawData,
    read_pos: usize,
}

pub struct JpegBitStreamReader<'a>
{
    data_ref: &'a JpegRawData,
    read_pos: usize,
    read_bitpos: usize,
    needs_escape: usize,
}

#[allow(dead_code)]
impl JpegRawData
{
    // Constructor
    pub fn new() -> Self
    {
        JpegRawData { data: Vec::new(), size: 0 }
    }

    pub fn read_from_file(&mut self, infilename: &String)
    {
        // ファイルのオープン
        let mut fp = File::open(infilename)
            .expect("File not found.");

        // ファイルからバイナリを読み込み
        self.data = Vec::new();
        fp.read_to_end(&mut self.data)
            .expect("File read error.");
        
        self.size = self.data.len();
    }

    pub fn get_size(&self) -> usize
    {
        return self.size;
    }

    pub fn read_u8(&self, pos: usize) -> Option<u8>
    {
        if pos >= self.size
        {
            return None;
        }
        else
        {
            return Some(self.data[pos]);
        }
    }

    pub fn read_u16be(&self, pos: usize) -> Option<u16>
    {
        if pos + 1 >= self.size
        {
            return None; 
        }
        else
        {
            let val: u16 = (self.data[pos] as u16) << 8
                         | self.data[pos+1] as u16;
            return Some(val);
        }
    }

    pub fn dump_binary(&self)
    {
        // 16進ダンプ
        for a in 0..self.size
        {
            print!("{:02x} ", &self.data[a]);
            if a % 16 == 15
            {
                println!();
            }
        }
    }
}

#[allow(dead_code)]
impl<'a> JpegReader<'a>
{
    pub fn new(data: &'a JpegRawData) -> Self
    {
        JpegReader{ data_ref: data, read_pos: 0 }
    }

    pub fn copy(&self) -> Self
    {
        JpegReader
        {
            data_ref: self.data_ref,
            read_pos: self.read_pos
        }
    }

    pub fn get_pos(&self) -> usize
    {
        return self.read_pos;
    }

    pub fn set_pos(&mut self, pos: usize)
    {
        self.read_pos = pos;
    }

    pub fn move_pos(&mut self, offset: isize)
    {
        let mut i = self.read_pos as isize + offset;
        if i < 0
        {
            i = 0;
        }
        self.read_pos = i as usize;
    }

    pub fn is_end(&self) -> bool
    {
        return self.read_pos >= self.data_ref.get_size();
    }

    pub fn read_u16be(&mut self) -> u16
    {
        let r = self.data_ref.read_u16be(self.read_pos);
        assert!(r.is_some());
        self.read_pos += 2;
        return r.unwrap();
    }

    pub fn read_u8(&mut self) -> u8
    {
        let r = self.data_ref.read_u8(self.read_pos);
        assert!(r.is_some());
        self.read_pos += 1;
        return r.unwrap();
    }
}

#[allow(dead_code)]
impl<'a> JpegBitStreamReader<'a>
{
    pub fn new(data: &'a JpegRawData) -> Self
    {
        JpegBitStreamReader
        {
            data_ref: data,
            read_pos: 0,
            read_bitpos: 0,
            needs_escape: 0,
        }
    }

    pub fn copy(&self) -> Self
    {
        JpegBitStreamReader
        {
            data_ref: self.data_ref,
            read_pos: self.read_pos,
            read_bitpos: self.read_bitpos,
            needs_escape: self.needs_escape,
        }
    }

    pub fn get_pos(&self) -> usize
    {
        return self.read_pos;
    }

    pub fn set_pos(&mut self, pos: usize, bitpos: usize)
    {
        self.read_pos = pos;
        self.read_bitpos = bitpos;
    }

    pub fn move_bitpos(&mut self, offset_bits: usize)
    {
        let mut i = self.read_bitpos + offset_bits;
        // Escape the 0x00 after 0xFF.
        if self.needs_escape > 0 && i >= self.needs_escape * 8
        {
            i += 8;
        }
        let j = i / 8 + self.read_pos;
        i = i & 7;
        self.read_pos = j;
        self.read_bitpos = i as usize;
        
        /*
        println!("Pos:{} Bit:{} {:08b} {:08b} {:08b}",
                  self.read_pos,
                  self.read_bitpos,
                  self.data_ref.read_u8(self.read_pos).unwrap(),
                  self.data_ref.read_u8(self.read_pos + 1).unwrap(),
                  self.data_ref.read_u8(self.read_pos + 2).unwrap(),
                );
        */
    }

    pub fn is_end(&self) -> bool
    {
        return self.read_pos >= self.data_ref.get_size();
    }

    pub fn read_bits16(&mut self) -> u16
    {
        let r0 = self.data_ref.read_u16be(self.read_pos);
        let r1 = self.data_ref.read_u8(self.read_pos + 2);
        assert!(r1.is_some());
        let mut b0 = r0.unwrap();
        let mut b1 = r1.unwrap();

        // Checking marker escape sequence
        self.needs_escape = 0;
        if b0 & 0xFF00 == 0xFF00
        {
            if b0 == 0xFF00 // Normal case where the stream contain 0xFF.
            {
                b0 = b0 | b1 as u16;
                self.needs_escape = 1;
            }
            else
            {
                println!("An unexpected marker detected: {:4x}", b0);
                // ToDo: exceptional sequence
            }
        }
        else if ( b0 & 0x00FF == 0x00FF ) && b1 == 0x00
        {
            self.needs_escape = 2;
        }

        if self.needs_escape > 0
        {
            //println!("**** Detected 0xFF followed by 0x00. Reading one more byte.");
            let r2 = self.data_ref.read_u8(self.read_pos + 3);
            assert!(r2.is_some());
            b1 = r2.unwrap();
        }

        // Align to the current bit position.
        b0 = b0 << self.read_bitpos;
        b1 = if self.read_bitpos == 0
        {
            0
        }
        else
        {
            b1 >> (8 - self.read_bitpos)
        };

        b0 | b1 as u16
    }

    pub fn check_marker(&mut self)
    {
        let offset = if self.read_bitpos == 0 { 0 } else { 1 };
        let r0 = self.data_ref.read_u16be(self.read_pos + offset);
        assert!(r0.is_some());
        let b0 = r0.unwrap();
        if b0 & 0xFF00 == 0xFF00
        {
            if b0 == 0xFF00
            {
                println!("**** Detected 0x00 after 0xFF.");
            }
            else
            {   
                println!("**** Detected a marker {:04x}", b0);
                self.read_pos += 2 + offset;
                self.read_bitpos = 0;
            }
        }
    }
}

//========================================================
