//! Mixer to mix all generated signals
pub struct MixerConfiguration {
    pub gain_voice_1: u8,
    pub gain_voice_2: u8,
    pub gain_voice_3: u8,
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