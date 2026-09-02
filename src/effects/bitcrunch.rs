//! Bitcrunch (chip-tune/8 bit) effect

use crate::effects::Effect;
use serde::{Deserialize, Serialize};

// 16-bit audio is reduced to 8-bit steps by quantizing to a 256-sample grid.
const QUANTIZE_STEP: i32 = 256;

// Effect settings: only toggles whether the crunching is active.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct BitcrunchConfiguration {
    pub enabled: bool,
}

// Bitcrunch effect instance storing its runtime configuration.
pub struct Bitcrunch {
    pub config: BitcrunchConfiguration,
}

impl Bitcrunch {
    pub fn new(config: BitcrunchConfiguration) -> Self {
        Bitcrunch { config }
    }

    // Replace the current settings without creating a new effect instance.
    pub fn reload(&mut self, config: BitcrunchConfiguration) {
        self.config = config;
    }
}

impl Effect for Bitcrunch {
    fn clock(&mut self, sample: i16) -> i16 {
        // When enabled, quantize the signal to 8-bit resolution by stepping it in multiples
        // of 256. This removes the lower 8 bits of amplitude information, creating a crunchy,
        // low-bit-depth sound while preserving the sample's sign and clamping to valid i16 range.
        if self.config.enabled {
            let quantized = (sample as i32 / QUANTIZE_STEP) * QUANTIZE_STEP;
            return quantized.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        }
        sample
    }
}
