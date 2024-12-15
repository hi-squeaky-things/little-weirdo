//! Math function optimized for embedded devices
/*
    Helper function to calculate the percentage of the given sample and keep the precision on the sample.
*/
pub fn percentage(sample: i16, percentage: i16) -> i16 {
    // scale up the sample (1000) to maintain precision of the sample when taking an percentage
    let sample_up_scale: i32 = sample as i32 * 1000;
    // calculate the percentage and scaled back down (by 1000) to maintain precision
    (sample_up_scale / 100 * percentage as i32 / 1000) as i16
}

const FX_SHIFT:u16 = 16;
  

pub fn ucfxmul(a: u16, b: u16) -> u32 {
    (a as u32* b as u32) >> self::FX_SHIFT
}

pub fn ifxmul(a: i32, b: u16) -> i32 {
    (a * b as i32) >> self::FX_SHIFT
}

pub fn fxmul( a: i64, b: i32) -> i64 {
    (a * b as i64) >> self::FX_SHIFT
}