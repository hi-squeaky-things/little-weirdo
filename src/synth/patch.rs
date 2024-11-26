use super::{envelope::Envelop, filter::FilterConfig, oscillator::Waveform};

pub struct Patch {
    pub voice_1: Waveform,
    pub voice_1_detune: i8,
    pub voice_1_mix_level: u8,
    pub voice_2: Waveform,
    pub voice_2_detune: i8,
    pub voice_2_mix_level: u8,
    pub lfo_1: u8,
    pub lfo_1_mix_level: u8,
    pub voice_1_env: Envelop,
    pub voice_2_env: Envelop,
    pub filter_config: FilterConfig,
    pub main_gain: u8,
    pub glide: bool,
    pub glide_rate: u8,
    pub mono: bool,
}
