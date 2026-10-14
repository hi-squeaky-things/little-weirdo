//! Mixer to mix all generated signals

use serde::{Deserialize, Serialize};

use super::AMOUNT_OF_VOICES;
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct MixerConfiguration {
    pub gain_voices: [u8; AMOUNT_OF_VOICES],
    pub gain_main: u8,
    #[serde(default = "default_balance")]
    pub balance: i8,
}


// -100 = left, 0 = middle, 100 = right
fn default_balance() -> i8 {
    0
}

pub struct Mixer {
    pub config: MixerConfiguration,
}

impl Mixer {
    pub fn new(config: MixerConfiguration) -> Self {
        Self {
            config,
        }
    }

    pub fn reload(&mut self, config: MixerConfiguration) {
        self.config = config;
    }


    pub fn apply_voice_gain(&self, sample: i16, voice_gain: u8) -> i16 {
        (sample as i64 * voice_gain as i64  / 1_000) as i16
    }
}
