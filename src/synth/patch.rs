//! Patch to config the sounds
 
use serde::Deserialize;
 use serde::Serialize;
use super::{effects::{filter::FilterConfig, overdrive::OverdriveConfiguration}, envelope::EnvelopConfiguration, mixer::MixerConfiguration,router::RoutingConfiguration, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig}, AMOUNT_OF_VOICES};


#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum SynthMode {
    Mono = 8 ,
    BiPoly = 4,
    QuadPoly = 2,
    OctoPoly = 1,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct SynthConfiguration {
    pub mode: SynthMode,
}


#[derive(Copy, Clone, Serialize, Deserialize)]
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
