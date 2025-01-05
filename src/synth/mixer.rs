//! Mixer to mix all generated signals

use super::AMOUNT_OF_VOICES;
#[derive(Copy, Clone)]
pub struct MixerConfiguration {
    pub gain_voices: [u8;AMOUNT_OF_VOICES],
    pub gain_main: u8,
}

pub struct Mixer {
    pub config: MixerConfiguration,
}


impl Mixer {
    pub fn new(config: MixerConfiguration) -> Self {
        Self {
            config
        }
    }

    pub fn reload(&mut self, config: MixerConfiguration) {
        self.config = config;
    }
}