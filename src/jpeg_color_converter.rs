//========================================================
//  jpeg_color_converter.rs
//
//========================================================

pub type ColorConvertFunc = fn(i16, i16, i16) -> (u8, u8, u8);

// No color conversion (pass-through)
#[allow(dead_code)]
pub fn pass_through_components(y: i16, cb: i16, cr: i16) -> (u8, u8, u8)
{
    (y as u8, cb as u8, cr as u8)
}

// YCbCr -> RGB : CCITT T.81 recommendation
//
// R = Y + 1.402 * (Cr - 128)
// G = Y - 0.34414 * (Cb - 128) - 0.71414 * (Cr - 128)
// B = Y + 1.772 * (Cb - 128)
#[allow(dead_code)]
pub fn ycbcr_to_rgb(y: i16, cb: i16, cr: i16) -> (u8, u8, u8)
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
#[allow(dead_code)]
pub fn ycbcr_to_rgb_bt601_fp(y: i16, cb: i16, cr: i16) -> (u8, u8, u8)
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



//========================================================
