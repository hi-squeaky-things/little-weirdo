//! Overdrive/Distortion effect
use crate::effects::Effect;
use serde::{Deserialize, Serialize};

// Different distortion curves applied once a signal exceeds the configured threshold.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum KindOfOverdrive {
    Hard,
    Soft,
    Softer,
}

// Runtime settings for the effect: when enabled and how aggressively it should clip.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct OverdriveConfiguration {
    pub threshold: i16,
    pub kind: KindOfOverdrive,
    pub enabled: bool,
}

// Effect instance holding the active configuration.
pub struct Overdrive {
    pub config: OverdriveConfiguration,
}

impl Overdrive {
    pub fn new(config: OverdriveConfiguration) -> Self {
        Self { config }
    }

    // Replace the current settings without creating a new effect instance.
    pub fn reload(&mut self, config: OverdriveConfiguration) {
        self.config = config;
    }
}

impl Effect for Overdrive {
    fn clock(&mut self, sample: i16) -> i16 {
        let threshold = self.config.threshold;
        let kind = self.config.kind;

        // Ignore the effect when disabled or when the input is still within the threshold.
        if !self.config.enabled || sample.abs() <= threshold {
            return sample;
        }

        // Keep the sample sign and calculate the absolute magnitude for clipping math.
        let sign = sample.signum();
        let magnitude = sample.abs() as i32;

        match kind {
            // Hard clipping: flatten anything above the threshold to a fixed ceiling.
            KindOfOverdrive::Hard => sign * threshold as i32 as i16,
            // Soft clipping: compress the excess gradually before the threshold is reached.
            KindOfOverdrive::Soft => {
                let value = ((magnitude - threshold as i32) / 2) + threshold as i32;
                sign * value as i16
            }
            // Softer clipping: more gentle compression than the standard soft mode.
            KindOfOverdrive::Softer => {
                let value = ((magnitude - threshold as i32) / 8) + threshold as i32;
                sign * value as i16
            }
        }
    }
}
