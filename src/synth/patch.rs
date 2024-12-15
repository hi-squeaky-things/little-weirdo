//! Patch to config the sounds
 
use super::{effects::{filter::FilterConfig, overdrive::OverdriveConfiguration}, envelope::EnvelopConfiguration, mixer::MixerConfiguration, patches::Patches, wavetable_oscillator::WaveTableOscillatorConfig, Synth, AMOUNT_OF_VOICE};

pub struct Patch {
    pub voices: [WaveTableOscillatorConfig;AMOUNT_OF_VOICE],
    pub envelops: [EnvelopConfiguration;AMOUNT_OF_VOICE],
    pub filter_config: FilterConfig,
    pub mixer_config: MixerConfiguration,
    pub overdrive_config: OverdriveConfiguration,    
}

impl Patch {
    pub fn default() -> Patch {
        Patches::get_patch(Patches::BassGuitar)
    }
}