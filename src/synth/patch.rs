//! Patch to config the sounds
 
use super::{effects::{filter::FilterConfig, overdrive::OverdriveConfiguration}, envelope::EnvelopConfiguration, mixer::MixerConfiguration,router::RoutingConfiguration, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig}, AMOUNT_OF_VOICE};
use bitflags::bitflags;

#[derive(Copy, Clone)]
pub struct Patch {
    pub voices: [WaveTableOscillatorConfig;AMOUNT_OF_VOICE],
    pub envelops: [EnvelopConfiguration;AMOUNT_OF_VOICE],
    pub lfo: WaveTableLoFreqOscillatorConfig,
    pub filter_config: FilterConfig,
    pub mixer_config: MixerConfiguration,
    pub overdrive_config: OverdriveConfiguration,   
    pub routering_config: RoutingConfiguration, 
}
