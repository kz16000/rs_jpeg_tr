//========================================================
//  jpeg_idct.rs
//
//========================================================
use std::f32::consts::PI;
use crate::jpeg_constants::JPEG_SAMPLE_BLOCK_SIZE;

#[allow(dead_code)]
pub struct JpegIdctManager
{
    tmp: [f32; JPEG_SAMPLE_BLOCK_SIZE],
}

#[allow(dead_code)]
impl JpegIdctManager
{
    // Constructor
    pub fn new() -> Self
    {
        JpegIdctManager
        {
            tmp: [0.0_f32; JPEG_SAMPLE_BLOCK_SIZE],
        }
    }

    // Non-optimized straight-forward implementation
    pub fn idct(mut self, coef: &mut [i16])
    {
        const FRAC_1_2: f32 = 0.5_f32;
        let frac_1_2sqrt2: f32 = FRAC_1_2 / 2.0_f32.sqrt();

        for y in 0..8
        {
            let fy = y as f32;
            for x in 0..8
            {
                let fx = x as f32;
                let mut val: f32 = 0.0_f32;
                for v in 0..8
                {
                    let cv = if v == 0 { frac_1_2sqrt2 } else { FRAC_1_2 };
                    for u in 0..8
                    {
                        let cu = if u == 0 { frac_1_2sqrt2 } else { FRAC_1_2 };
                        val += cu * cv * (coef[v*8 + u] as f32)
                            * ((2.0_f32 * fx + 1.0_f32) * u as f32 * PI / 16.0_f32 ).cos()
                            * ((2.0_f32 * fy + 1.0_f32) * v as f32 * PI / 16.0_f32 ).cos()
                    }
                }
                self.tmp[y*8 + x] = val;
            }
        }

        for i in 0..JPEG_SAMPLE_BLOCK_SIZE
        {
            coef[i] = self.tmp[i] as i16;
        }
    }
}

//========================================================
