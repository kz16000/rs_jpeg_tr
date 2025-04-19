//========================================================
//  jpeg_sampler.rs
//
//========================================================
use crate::jpeg_sample_block::JpegSampleBlock;
use crate::jpeg_color_converter::ColorConvertFunc;
use crate::jpeg_color_converter;

#[allow(dead_code)]
pub enum JpegSampleMode
{
    JpegSampleMode444,
    JpegSampleMode422,
    JpegSampleMode440,
    JpegSampleMode420,
    JpegSampleModeNone,
}

type UpsamplerFunc = fn(&[JpegSampleBlock], ColorConvertFunc, &mut[u8]);

#[allow(dead_code)]
pub struct JpegSampler
{
    upsampling_func: UpsamplerFunc,
    color_convert_func: ColorConvertFunc,
}

macro_rules! put_pixel
{
    ($out_buf:expr, $i:expr, $ccv:expr, $stride:expr) =>
    {
        ($out_buf[$i], $out_buf[$i+1], $out_buf[$i+2]) = $ccv;
        $i += $stride;
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
            upsampling_func: Self::upsampling1,
            color_convert_func: jpeg_color_converter::ycbcr_to_rgb,
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
            JpegSampleMode::JpegSampleMode420 => self.upsampling_func = Self::upsampling420,
            _ => self.upsampling_func = Self::upsampling1,
        }
    }

    // Calling the current upsampler
    pub fn upsampling(&self, blocks: &[JpegSampleBlock], out_buf: &mut[u8])
    {
        (self.upsampling_func)(blocks, self.color_convert_func, out_buf);
    }

    // For mono component (no upsampling)
    fn upsampling1(blocks: &[JpegSampleBlock], convert_func: ColorConvertFunc, out_buf: &mut[u8])
    {
        let c_stride = 3;
        let lf_stride = 0;
        let mut i = 0;
        let mut t = 0;
        for y in blocks[0].iter()
        {
            put_pixel!(out_buf, i, convert_func(*y, 0x80, 0x80), c_stride);
            t = (t + 1) & 7;
            if t == 0
            {
                i += lf_stride;
            }
        }
    }

    // For 4:4:4 (no upsampling)
    fn upsampling444(blocks: &[JpegSampleBlock], convert_func: ColorConvertFunc, out_buf: &mut[u8])
    {
        let c_stride = 3;
        let lf_stride = 12;
        let mut i = 0;
        let mut t = 0;
        let iter_y = blocks[0].iter();
        let iter_cb = blocks[1].iter();
        let iter_cr = blocks[2].iter();
        for (y, (cb, cr)) in iter_y.zip(iter_cb.zip(iter_cr))
        {
            put_pixel!(out_buf, i, convert_func(*y, *cb, *cr), c_stride);
            t = (t + 1) & 7;
            if t == 0
            {
                i += lf_stride;
            }
        }
    }

    // Up-sampling for 4:2:2
    fn upsampling422(blocks: &[JpegSampleBlock], convert_func: ColorConvertFunc, out_buf: &mut[u8])
    {
        let c_stride = 3;
        let lf_stride = 12;
        let mut i = 0;
        let mut t = 0;
        let mut iter_y0 = blocks[0].iter();
        let mut iter_y1 = blocks[1].iter();
        let iter_cb = blocks[2].iter();
        let iter_cr = blocks[3].iter();
        for (cb, cr) in iter_cb.zip(iter_cr)
        {
            if t < 4
            {
                let y0 = iter_y0.next().unwrap();
                put_pixel!(out_buf, i, convert_func(*y0, *cb, *cr), c_stride);
                let y0 = iter_y0.next().unwrap();
                put_pixel!(out_buf, i, convert_func(*y0, *cb, *cr), c_stride);
            }
            else
            {
                let y1 = iter_y1.next().unwrap();
                put_pixel!(out_buf, i, convert_func(*y1, *cb, *cr), c_stride);
                let y1 = iter_y1.next().unwrap();
                put_pixel!(out_buf, i, convert_func(*y1, *cb, *cr), c_stride);
            }
            t = (t + 1) & 7;
            if t == 0
            {
                i += lf_stride;
            }
        }
    }

    // Up-sampling for 440
    fn upsampling440(blocks: &[JpegSampleBlock], convert_func: ColorConvertFunc, out_buf: &mut[u8])
    {
        let c_stride = 3;
        let lf_stride = 12;
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
                    put_pixel!(out_buf, i, convert_func(*y, *cb, *cr), c_stride);
                }
                else
                {
                    let cb = iter_cb1.next().unwrap();
                    let cr = iter_cr1.next().unwrap();
                    put_pixel!(out_buf, i, convert_func(*y, *cb, *cr), c_stride);
                }
                t = (t + 1) & 15;
                if t & 7 == 0
                {
                    i += lf_stride;
                }   
            }
        }
    }

    // Up-sampling for 420
    fn upsampling420(blocks: &[JpegSampleBlock], convert_func: ColorConvertFunc, out_buf: &mut[u8])
    {
        let c_stride = 3;
        let lf_stride = 12;
        let mut i = 0;
        let mut iter_cb0 = blocks[4].iter();
        let mut iter_cb1 = iter_cb0.clone();
        let mut iter_cr0 = blocks[5].iter();
        let mut iter_cr1 = iter_cr0.clone();
        for x in 0..2
        {
            let mut iter_y0 = blocks[x*2].iter();
            let mut iter_y1 = blocks[x*2+1].iter();
            for t in 0..64
            {
                let cb = (if t & 8 == 0 { iter_cb0.next() } else { iter_cb1.next() }).unwrap();
                let cr = (if t & 8 == 0 { iter_cr0.next() } else { iter_cr1.next() }).unwrap();
                
                if t & 4 == 0
                {
                    let y0 = iter_y0.next().unwrap();
                    put_pixel!(out_buf, i, convert_func(*y0, *cb, *cr), c_stride);
                    let y0 = iter_y0.next().unwrap();
                    put_pixel!(out_buf, i, convert_func(*y0, *cb, *cr), c_stride);
                }
                else
                {
                    let y1 = iter_y1.next().unwrap();
                    put_pixel!(out_buf, i, convert_func(*y1, *cb, *cr), c_stride);
                    let y1 = iter_y1.next().unwrap();
                    put_pixel!(out_buf, i, convert_func(*y1, *cb, *cr), c_stride);
                }

                if t & 7 == 7
                {
                    i += lf_stride;
                }   
            }
        }
    }
}


//========================================================
