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
  

/// Multiply two values and shift the result by FX_SHIFT bits.
///
/// # Arguments
///
/// * `a` - The first value to multiply.
/// * `b` - The second value to multiply.
///
/// # Returns
///
/// A 32-bit integer representing the shifted product of `a` and `b`.
pub fn ucfx_mul(a: u16, b: u16) -> u32 {
    (a as u32 * b as u32) >> self::FX_SHIFT
}

/// Multiply two values and shift the result by FX_SHIFT bits.
///
/// # Arguments
///
/// * `a` - The first value to multiply. Must be a 32-bit integer or smaller.
/// * `b` - The second value to multiply. Can be up to 16 bits wide.
///
/// # Returns
///
/// A signed 32-bit integer representing the shifted product of `a` and `b`.
pub fn ifx_mul(a: i32, b: u16) -> i32 {
    (a * b as i32) >> self::FX_SHIFT
}

/// Multiply two values and shift the result by FX_SHIFT bits.
///
/// # Arguments
///
/// * `a` - The first value to multiply. Can be up to 64 bits wide.
/// * `b` - The second value to multiply. Must be a 32-bit integer or smaller.
///
/// # Returns
///
/// A signed 64-bit integer representing the shifted product of `a` and `b`.
pub fn fx_mul(a: i64, b: i32) -> i64 {
    (a * b as i64) >> self::FX_SHIFT
}