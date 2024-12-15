//! Mixer to mix all generated signals
pub struct MixerConfiguration {
    pub gain_voices: [u8;3],
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