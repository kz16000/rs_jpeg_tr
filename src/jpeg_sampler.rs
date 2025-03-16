//========================================================
//  jpeg_sampler.rs
//
//========================================================
use crate::jpeg_sample_block::JpegSampleBlock;

#[allow(dead_code)]
pub struct JpegSampler444
{

}

#[allow(dead_code)]
pub struct JpegSampler422
{

}

#[allow(dead_code)]
impl JpegSampler444
{
    // Constructor
    pub fn new() -> Self
    {
        JpegSampler444{}
    }

    // Perform Up-sampling
    pub fn upsampling(&self, blocks: &[JpegSampleBlock], out_buf: &mut[u8])
    {
        let mut i = 0;
        let iter_y = blocks[0].iter();
        let iter_cb = blocks[1].iter();
        let iter_cr = blocks[2].iter();
        for (y, (cb, cr)) in iter_y.zip(iter_cb.zip(iter_cr))
        {
            out_buf[i] = *y as u8;
            out_buf[i+1] = *cb as u8;
            out_buf[i+2] = *cr as u8;
            i += 3;
        }
    }
}

#[allow(dead_code)]
impl JpegSampler422
{
    // Constructor
    pub fn new() -> Self
    {
        JpegSampler422{}
    }

    // Perform Up-sampling
    pub fn upsampling(&self, blocks: &[JpegSampleBlock], out_buf: &mut[u8])
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
                out_buf[i] = *y0 as u8;
                out_buf[i+1] = *cb as u8;
                out_buf[i+2] = *cr as u8;
                i += 3;

                let y0 = iter_y0.next().unwrap();
                out_buf[i] = *y0 as u8;
                out_buf[i+1] = *cb as u8;
                out_buf[i+2] = *cr as u8;
                i += 3;
            }
            else
            {
                let y1 = iter_y1.next().unwrap();
                out_buf[i] = *y1 as u8;
                out_buf[i+1] = *cb as u8;
                out_buf[i+2] = *cr as u8;
                i += 3;

                let y1 = iter_y1.next().unwrap();
                out_buf[i] = *y1 as u8;
                out_buf[i+1] = *cb as u8;
                out_buf[i+2] = *cr as u8;
                i += 3;
            }
            t = (t + 1) & 7;
        }
    }
}

//========================================================
