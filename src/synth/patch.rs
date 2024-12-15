//! Patch to config the sounds
 
use super::{effects::{filter::FilterConfig, overdrive::OverdriveConfiguration}, envelope::EnvelopConfiguration, mixer::MixerConfiguration, patches::Patches, wavetable_oscillator::WaveTableOscillatorConfig};

pub struct Patch {
    pub voices: [WaveTableOscillatorConfig;3],
    pub envelops: [EnvelopConfiguration;3],

 
    /// Filter configuration, including cutoff and resonance
    pub filter_config: FilterConfig,
    pub mixer_config: MixerConfiguration,
    pub overdrive_config: OverdriveConfiguration,    
}

impl Patch {
    pub fn default() -> Patch {
        let patch: Patch = Patches::get_patch(Patches::BassGuitar);
        patch
    }
}