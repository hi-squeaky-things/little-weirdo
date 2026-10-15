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

pub(crate) fn envelope_level(sample: i16, level: u32) -> i16 {
    (sample as i32 * level as i32 / (1 << 16)) as i16
}

const FX_SHIFT: u16 = 16;

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

#[cfg(test)]
mod tests {
    use super::{envelope_level, percentage};

    const LEVEL_SCALE: u32 = 1 << 16;

    #[test]
    fn envelope_scaler_handles_silence_full_level_and_signed_extrema() {
        assert_eq!(envelope_level(i16::MAX, 0), 0);
        assert_eq!(envelope_level(i16::MIN, 0), 0);
        assert_eq!(envelope_level(i16::MAX, LEVEL_SCALE), i16::MAX);
        assert_eq!(envelope_level(i16::MIN, LEVEL_SCALE), i16::MIN);
    }

    #[test]
    fn envelope_scaler_matches_whole_percent_levels() {
        for level_percentage in [0, 25, 50, 75, 100] {
            let level = level_percentage as u32 * LEVEL_SCALE / 100;
            let expected = (30_000_i32 * level_percentage / 100) as i16;
            assert_eq!(envelope_level(30_000, level), expected);
        }
    }

    #[test]
    fn fractional_envelope_level_preserves_sub_percent_gain() {
        let fractional_level = LEVEL_SCALE * 50_500 / 100_000;
        assert!(envelope_level(30_000, fractional_level) > percentage(30_000, 50));
        assert!(envelope_level(-30_000, fractional_level) < percentage(-30_000, 50));
    }
}
