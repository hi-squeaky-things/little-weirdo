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
