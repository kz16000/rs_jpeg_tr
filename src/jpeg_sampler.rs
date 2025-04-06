//========================================================
//  jpeg_sampler.rs
//
//========================================================
use crate::jpeg_sample_block::JpegSampleBlock;

#[allow(dead_code)]
pub enum JpegSampleMode
{
    JpegSampleMode444,
    JpegSampleMode422,
    JpegSampleMode440,
    JpegSampleMode420,
    JpegSampleModeNone,
}

#[allow(dead_code)]
pub struct JpegSampler
{
    upsampling_func: fn(&[JpegSampleBlock], &mut[u8]),
}

macro_rules! put_pixel
{
    ($out_buf:expr, $i:expr, $y:expr, $cb:expr, $cr:expr) =>
    {
        ($out_buf[$i], $out_buf[$i+1], $out_buf[$i+2]) = ($y, $cb, $cr);
        $i += 3;
    };
}

#[allow(dead_code)]
impl JpegSampler
{
    // constructor
    pub fn new() -> Self
    {
        JpegSampler
        {
            upsampling_func: Self::upsampling444,
        }
    }

    // Sets the sampler function
    pub fn set_sampling_mode(&mut self, mode: JpegSampleMode)
    {
        match mode
        {
            JpegSampleMode::JpegSampleMode444 => self.upsampling_func = Self::upsampling444,
            JpegSampleMode::JpegSampleMode422 => self.upsampling_func = Self::upsampling422,
            JpegSampleMode::JpegSampleMode440 => self.upsampling_func = Self::upsampling440,
            JpegSampleMode::JpegSampleMode420 => println!("set_sampling_mode: 420 not supported yet."),
            _ => self.upsampling_func = Self::upsampling1,
        }
    }

    // Calling the current upsampler
    pub fn upsampling(&self, blocks: &[JpegSampleBlock], out_buf: &mut[u8])
    {
        (self.upsampling_func)(blocks, out_buf);
    }

    // For mono component (no upsampling)
    fn upsampling1(blocks)
    {
        let mut i = 0;
        for y in blocks[0]
        {
            put_pixel!(out_buf, i, *y as u8, *cb as u8, *cr as u8);
        }
    }

    // For 4:4:4 (no upsampling)
    fn upsampling444(blocks: &[JpegSampleBlock], out_buf: &mut[u8])
    {
        let mut i = 0;
        let iter_y = blocks[0].iter();
        let iter_cb = blocks[1].iter();
        let iter_cr = blocks[2].iter();
        for (y, (cb, cr)) in iter_y.zip(iter_cb.zip(iter_cr))
        {
            put_pixel!(out_buf, i, *y as u8, *cb as u8, *cr as u8);
        }
    }

    // Up-sampling for 4:2:2
    fn upsampling422(blocks: &[JpegSampleBlock], out_buf: &mut[u8])
    {
        let mut i = 0;
        let mut iter_y0 = blocks[0].iter();
        let mut iter_y1 = blocks[1].iter();
        let iter_cb = blocks[2].iter();
        let iter_cr = blocks[3].iter();
        let mut t = 0;
        for (cb, cr) in iter_cb.zip(iter_cr)
        {
            if t < 4
            {
                let y0 = iter_y0.next().unwrap();
                put_pixel!(out_buf, i, *y0 as u8, *cb as u8, *cr as u8);
                let y0 = iter_y0.next().unwrap();
                put_pixel!(out_buf, i, *y0 as u8, *cb as u8, *cr as u8);
            }
            else
            {
                let y1 = iter_y1.next().unwrap();
                put_pixel!(out_buf, i, *y1 as u8, *cb as u8, *cr as u8);
                let y1 = iter_y1.next().unwrap();
                put_pixel!(out_buf, i, *y1 as u8, *cb as u8, *cr as u8);
            }
            t = (t + 1) & 7;
        }
    }

    // Up-sampling for 440
    fn upsampling440(blocks: &[JpegSampleBlock], out_buf: &mut[u8])
    {
        let mut i = 0;
        let mut t = 0;
        let mut iter_cb0 = blocks[2].iter();
        let mut iter_cb1 = iter_cb0.clone();
        let mut iter_cr0 = blocks[3].iter();
        let mut iter_cr1 = iter_cr0.clone();
        for x in 0..2
        {
            let iter_y = blocks[x].iter();
            for y in iter_y
            {
                if t < 8
                {
                    let cb = iter_cb0.next().unwrap();
                    let cr = iter_cr0.next().unwrap();
                    put_pixel!(out_buf, i, *y as u8, *cb as u8, *cr as u8);
                }
                else
                {
                    let cb = iter_cb1.next().unwrap();
                    let cr = iter_cr1.next().unwrap();
                    put_pixel!(out_buf, i, *y as u8, *cb as u8, *cr as u8);
                }
                t = (t + 1) & 15;
            }
        }
    }
}


//========================================================
