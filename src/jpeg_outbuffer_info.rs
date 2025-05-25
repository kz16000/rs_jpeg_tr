//========================================================
//  jpeg_outbuffer_info.rs
//
//========================================================

pub struct JpegOutBufferInfo
{
    width: u16,
    height: u16,
    bpp: u8,
}

#[allow(dead_code)]
impl JpegOutBufferInfo
{
    // Constructor
    pub fn new() -> Self
    {
        JpegOutBufferInfo
        {
            width: 0,
            height: 0,
            bpp: 3,
        }
    }

    // Sets parmeters
    pub fn set_parameters(&mut self, width: usize, height: usize, bpp: usize)
    {
        self.width = width as u16;
        self.height = height as u16;
        self.bpp = bpp as u8;
    }

    // Gets width
    pub fn get_width(&self) -> usize
    {
        self.width as usize
    }

    // Gets height
    pub fn get_height(&self) -> usize
    {
        self.height as usize
    }

    // Gets width/height
    pub fn get_dimension(&self) -> (usize, usize)
    {
        (self.width as usize, self.height as usize)
    }

    // Gets bpp
    pub fn get_bpp(&self) -> usize
    {
        self.bpp as usize
    }

    // Gets total buffer size
    pub fn get_total_buffer_size(&self) -> usize
    {
        self.width as usize * self.height as usize * self.bpp as usize
    }

}

//========================================================
