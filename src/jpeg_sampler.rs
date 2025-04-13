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

type ColorConvertFunc = fn(i16, i16, i16) -> (u8, u8, u8);
type UpsamplerFunc = fn(&[JpegSampleBlock], ColorConvertFunc, &mut[u8]);

#[allow(dead_code)]
pub struct JpegSampler
{
    upsampling_func: UpsamplerFunc,
    color_convert_func: ColorConvertFunc,
}

macro_rules! put_pixel
{
    ($out_buf:expr, $i:expr, $ccv:expr) =>
    {
        ($out_buf[$i], $out_buf[$i+1], $out_buf[$i+2]) = $ccv;
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
            upsampling_func: Self::upsampling1,
            color_convert_func: Self::ycbcr_to_rgb,
        }
    }

    // No color conversion (pass-through)
    fn pass_through_components(y: i16, cb: i16, cr: i16) -> (u8, u8, u8)
    {
        (y as u8, cb as u8, cr as u8)
    }

    // YCbCr -> RGB : CCITT T.81 recommendation
    //
    // R = Y + 1.402 * (Cr - 128)
    // G = Y - 0.34414 * (Cb - 128) - 0.71414 * (Cr - 128)
    // B = Y + 1.772 * (Cb - 128)
    fn ycbcr_to_rgb(y: i16, cb: i16, cr: i16) -> (u8, u8, u8)
    {
        let ly = (y as i32) << 16;
        let lcb = ((cb as i32) << 8) - 0x8000;
        let lcr = ((cr as i32) << 8) - 0x8000;

        let mut r = ly + 359 * lcr;
        let mut g = ly - 88 * lcb - 183 * lcr;
        let mut b = ly + 454 * lcb;

        r = r.clamp(0, 0xFFFFFF);
        g = g.clamp(0, 0xFFFFFF);
        b = b.clamp(0, 0xFFFFFF);
       
        ((r >> 16) as u8, (g >> 16) as u8, (b >> 16) as u8)
    }

    // YCbCr -> RGB : ITU-R BT.601 precise FP ver
    //
    // R = Y + 1.402   * (Cr - 128)
    // G = Y - 0.344136 * (Cb - 128) - 0.714136 * (Cr - 128)
    // B = Y + 1.772   * (Cb - 128)
    fn ycbcr_to_rgb_bt601_fp(y: i16, cb: i16, cr: i16) -> (u8, u8, u8)
    {
        let fy = y as f32;
        let fcb = cb as f32;
        let fcr = cr as f32;

        let r = fy + 1.402 * (fcr - 128.0);
        let g = fy - 0.344136 * (fcb - 128.0) - 0.714136 * (fcr - 128.0);
        let b = fy + 1.772 * (fcb - 128.0);
        
        (r as u8, g as u8, b as u8)
    }

    // For limited range
    //
    // Y_limited = Y * 219 + 16
    // Cb_limited = Cb * 224 + 128
    // Cr_limited = Cr * 224 + 128

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
        let mut i = 0;
        for y in blocks[0].iter()
        {
            put_pixel!(out_buf, i, convert_func(*y, 0x80, 0x80));
        }
    }

    // For 4:4:4 (no upsampling)
    fn upsampling444(blocks: &[JpegSampleBlock], convert_func: ColorConvertFunc, out_buf: &mut[u8])
    {
        let mut i = 0;
        let iter_y = blocks[0].iter();
        let iter_cb = blocks[1].iter();
        let iter_cr = blocks[2].iter();
        for (y, (cb, cr)) in iter_y.zip(iter_cb.zip(iter_cr))
        {
            put_pixel!(out_buf, i, convert_func(*y, *cb, *cr));
        }
    }

    // Up-sampling for 4:2:2
    fn upsampling422(blocks: &[JpegSampleBlock], convert_func: ColorConvertFunc, out_buf: &mut[u8])
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
                put_pixel!(out_buf, i, convert_func(*y0, *cb, *cr));
                let y0 = iter_y0.next().unwrap();
                put_pixel!(out_buf, i, convert_func(*y0, *cb, *cr));
            }
            else
            {
                let y1 = iter_y1.next().unwrap();
                put_pixel!(out_buf, i, convert_func(*y1, *cb, *cr));
                let y1 = iter_y1.next().unwrap();
                put_pixel!(out_buf, i, convert_func(*y1, *cb, *cr));
            }
            t = (t + 1) & 7;
        }
    }

    // Up-sampling for 440
    fn upsampling440(blocks: &[JpegSampleBlock], convert_func: ColorConvertFunc, out_buf: &mut[u8])
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
                    put_pixel!(out_buf, i, convert_func(*y, *cb, *cr));
                }
                else
                {
                    let cb = iter_cb1.next().unwrap();
                    let cr = iter_cr1.next().unwrap();
                    put_pixel!(out_buf, i, convert_func(*y, *cb, *cr));
                }
                t = (t + 1) & 15;
            }
        }
    }

    // Up-sampling for 420
    fn upsampling420(blocks: &[JpegSampleBlock], convert_func: ColorConvertFunc, out_buf: &mut[u8])
    {
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
                    put_pixel!(out_buf, i, convert_func(*y0, *cb, *cr));
                    let y0 = iter_y0.next().unwrap();
                    put_pixel!(out_buf, i, convert_func(*y0, *cb, *cr));
                }
                else
                {
                    let y1 = iter_y1.next().unwrap();
                    put_pixel!(out_buf, i, convert_func(*y1, *cb, *cr));
                    let y1 = iter_y1.next().unwrap();
                    put_pixel!(out_buf, i, convert_func(*y1, *cb, *cr));
                }
            }
        }
    }
}


//========================================================
