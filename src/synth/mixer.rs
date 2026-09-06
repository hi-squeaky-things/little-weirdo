//! Mixer to mix all generated signals

use serde::{Deserialize, Serialize};

use super::AMOUNT_OF_VOICES;
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct MixerConfiguration {
    pub gain_voices: [u8; AMOUNT_OF_VOICES],
    pub gain_main: u8,
    #[serde(default = "default_avc_enabled")]
    pub avc_enabled: bool,
    #[serde(default = "default_avc_decay_time_ms")]
    pub avc_decay_time_ms: u16,
}

fn default_avc_enabled() -> bool {
    true
}

fn default_avc_decay_time_ms() -> u16 {
    50
}

pub struct Mixer {
    pub config: MixerConfiguration,
    avc_gain: u32,
}

impl Mixer {
    pub fn new(config: MixerConfiguration) -> Self {
        Self {
            config,
            avc_gain: 100_000,
        }
    }

    pub fn reload(&mut self, config: MixerConfiguration) {
        self.config = config;
    }

    pub fn update_avc(&mut self, active_notes: usize, sample_rate: u16) {
        let target_gain = if active_notes == 0 {
            0
        } else if self.config.avc_enabled {
            100_000 / active_notes as u32
        } else {
            100_000
        };

        if self.config.avc_decay_time_ms == 0 {
            self.avc_gain = target_gain;
            return;
        }

        let decay_samples = (self.config.avc_decay_time_ms as u32
            * sample_rate.max(1) as u32
            / 1_000)
            .max(1);
        let step = (100_000 / decay_samples).max(1);

        if self.avc_gain < target_gain {
            self.avc_gain = (self.avc_gain + step).min(target_gain);
        } else if self.avc_gain > target_gain {
            self.avc_gain = self.avc_gain.saturating_sub(step).max(target_gain);
        }
    }

    pub fn apply_voice_gain(&self, sample: i16, voice_gain: u8) -> i16 {
        (sample as i64 * voice_gain as i64 * self.avc_gain as i64 / 10_000_000) as i16
    }

}
