//! Patch to config the sounds
 
use super::{effects::{filter::FilterConfig, overdrive::{self, KindOfOverdrive}}, envelope::Envelop, oscillator::Waveform, patches::Patches};

pub struct Patch {
    /// The waveform for voice one, this is one of the available WaveTable waveforms
    pub voice_1: Waveform,
    /// Detune value for voice 1 (range -100 to 100)
    pub voice_1_detune: i8,
    /// Mix level for voice 1 (range 0 to 127)
    pub voice_1_mix_level: u8,
    /// The waveform for voice two, this is one of the available WaveTable waveforms
    pub voice_2: Waveform,
    /// Detune value for voice 2 (range -100 to 100)
    pub voice_2_detune: i8,
    /// Mix level for voice 2 (range 0 to 127)
    pub voice_2_mix_level: u8,
    /// LFO frequency
    pub lfo_1: u8,
    /// Level of the first LFO modulation signal (range 0 to 100%)
    pub lfo_1_mix_level: u8,
    /// Envelope settings for voice 1
    pub voice_1_env: Envelop,
    /// Envelope settings for voice 2
    pub voice_2_env: Envelop,
    /// Filter configuration, including cutoff and resonance
    pub filter_config: FilterConfig,
    /// Overall main volume level (range 0 to 127)
    pub main_gain: u8,
    /// Glide function enabled or disabled
    pub glide: bool,
    /// How fast you glide from one note to another (in Ms)
    pub glide_rate: u8,
    /// Mono mode enabled or disabled, default is false
    pub mono: bool,
    /// Mono mode enabled or disabled, default is false
    pub overdrive: bool,
    pub overdrive_mode: KindOfOverdrive,    
}

impl Patch {
    pub fn default() -> Patch {
        let patch: Patch = Patches::get_patch(Patches::BassGuitar);
        patch
    }
}