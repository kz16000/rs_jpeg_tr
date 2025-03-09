//========================================================
//  jpeg_idct.rs
//
//========================================================
//use std::f32::consts::PI;
use crate::jpeg_constants::JPEG_SAMPLE_BLOCK_SIZE;

#[allow(dead_code)]
pub struct JpegIdctManager
{
    tmp: [f32; JPEG_SAMPLE_BLOCK_SIZE],
}

#[allow(dead_code)]
impl JpegIdctManager
{
    // Constants
    const FRAC_1_2: f32 = 0.5_f32;
    const FRAC_1_2SQRT2: f32 = 0.35355338_f32;
    /*
    const COS_TABLE_SIZE: usize = 16;
    const COS_TABLE: [f32; Self::COS_TABLE_SIZE] =
    [
         1.0,         0.98078525,   0.9238795,   0.8314696,
         0.70710677,  0.5555702,    0.38268343,  0.19509023,
         0.0,        -0.19509032,  -0.38268352, -0.55557036,
        -0.70710677, -0.83146966,  -0.9238796,  -0.9807853,
    ];
    */
    const COS_TABLE_SIZE: usize = 8;
    const COS_TABLE: [f32; Self::COS_TABLE_SIZE] =
    [
         1.0,         0.98078525,   0.92387955,  0.83146963,
         0.70710677,  0.55557028,   0.38268348,  0.19509028,
    ];   

    // Constructor
    pub fn new() -> Self
    {
        JpegIdctManager
        {
            tmp: [0.0_f32; JPEG_SAMPLE_BLOCK_SIZE],
        }
    }

    // Offset and clamp function to fit f32 value into range (0-255)
    fn offset_and_clamp(val: f32) -> i16
    {
        let i: i16 = (val + 128.5_f32) as i16;
        if i < 0
        {
            0
        }
        else if i > 255
        {
            255
        } 
        else
        {
            i
        }
    }

    // Tabled version of discrete cos(i * PI /16)
    fn lookup_tabled_cos(&self, idx: usize) -> f32
    {
        let mut i = idx & (Self::COS_TABLE_SIZE - 1);
        let mut sign = if idx & (Self::COS_TABLE_SIZE << 1) == 0 { 1.0_f32 } else { -1.0_f32 };
        if idx & Self::COS_TABLE_SIZE != 0
        {
            if i == 0
            {
                return 0.0_f32;
            }
            else
            {
                i = Self::COS_TABLE_SIZE - i;
                sign = -sign;
            }
        }
        Self::COS_TABLE[i] * sign
    }

    // Non-optimized straight-forward implementation
    pub fn idct(&mut self, coef: &mut [i16])
    {
        for y in 0..8
        {
            for x in 0..8
            {
                let mut val: f32 = 0.0_f32;
                for v in 0..8
                {
                    let cv = if v == 0 { Self::FRAC_1_2SQRT2 } else { Self::FRAC_1_2 };
                    for u in 0..8
                    {
                        let cu = if u == 0 { Self::FRAC_1_2SQRT2 } else { Self::FRAC_1_2 };
                        val += cu * cv * (coef[v*8 + u] as f32)
                            * self.lookup_tabled_cos( (x * 2 + 1) * u)
                            * self.lookup_tabled_cos( (y * 2 + 1) * v);
                    }
                }
                self.tmp[y*8 + x] = val;
            }
        }

        for i in 0..JPEG_SAMPLE_BLOCK_SIZE
        {
            coef[i] = Self::offset_and_clamp(self.tmp[i]);
        }
    }

    // For debug
    pub fn dump_cos_table(&self)
    {
        for i in 0..32
        {
            let v = self.lookup_tabled_cos(i);
            println!("cos(i={}): {}", i, v);
        }  
    }
}

//========================================================
