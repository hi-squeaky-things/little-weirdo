//! Patch to config the sounds
 
use super::{effects::{filter::FilterConfig, overdrive::OverdriveConfiguration}, envelope::EnvelopConfiguration, mixer::MixerConfiguration,router::RoutingConfiguration, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig}, AMOUNT_OF_VOICES};
use bitflags::bitflags;


#[derive(Copy, Clone, PartialEq)]
pub enum SynthMode {
    Mono,
    BiPoly,
    QuadPoly,
    OctoPoly
}

#[derive(Copy, Clone)]
pub struct SynthConfiguration {
    pub mode: SynthMode,
}


#[derive(Copy, Clone)]
pub struct Patch {
    pub voices: [WaveTableOscillatorConfig;AMOUNT_OF_VOICES],
    pub envelops: [EnvelopConfiguration;AMOUNT_OF_VOICES],
    pub lfos: [WaveTableLoFreqOscillatorConfig;AMOUNT_OF_VOICES / 2],
    pub filter_config: FilterConfig,
    pub mixer_config: MixerConfiguration,
    pub overdrive_config: OverdriveConfiguration,   
    pub routering_config: RoutingConfiguration, 
    pub synth_config: SynthConfiguration,
}
