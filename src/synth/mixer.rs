//! Mixer to mix all generated signals
pub struct Mixer {
    pub gain_voice_1: u8,
    pub gain_voice_2: u8,
    pub gain_lfo_1: u8,
    pub gain_main: u8,
    pub velocity: u8,
}

impl Mixer {
    pub fn new(gain_voice_1: u8, gain_voice_2: u8, gain_lfo_1: u8, gain_main: u8) -> Self {
        Self {
          gain_voice_1,
          gain_voice_2,
          gain_lfo_1,
          gain_main,
          velocity: 0,
        }
    }
}