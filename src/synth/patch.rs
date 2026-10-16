//! Patch to config the sounds
extern crate alloc;
use alloc::string::String;

use crate::effects::{delay::DelayConfiguration, flanger::FlangerConfiguration};

use super::effects::bitcrunch::BitcrunchConfiguration;
use super::{
    effects::{filter::FilterConfig, overdrive::OverdriveConfiguration},
    envelope::EnvelopConfiguration,
    mixer::MixerConfiguration,
    router::RoutingConfiguration,
    waveform_oscillator::{WaveformLFOConfig, WaveformOscillatorConfig},
    AMOUNT_OF_VOICES,
};
use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct FmConfiguration {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_modulator_waveform")]
    pub modulator_waveform: u8,
    #[serde(default = "default_modulator_ratio")]
    pub modulator_ratio: u8,
    #[serde(default = "default_modulation_depth_hz")]
    pub modulation_depth_hz: u16,
    #[serde(default = "default_carrier_waveform")]
    pub carrier_waveform: u8,
}

impl Default for FmConfiguration {
    fn default() -> Self {
        Self {
            enabled: false,
            carrier_waveform: default_carrier_waveform(),
            modulator_waveform: default_modulator_waveform(),
            modulator_ratio: default_modulator_ratio(),
            modulation_depth_hz: default_modulation_depth_hz(),
        }
    }
}

fn default_carrier_waveform() -> u8 {
    0
}

fn default_modulator_waveform() -> u8 {
    0
}

fn default_modulator_ratio() -> u8 {
    1
}

fn default_modulation_depth_hz() -> u16 {
    100
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum SynthMode {
    Mono = 1,
    BiPoly = 2,
    QuadPoly = 4,
    OctoPoly = 8,
}

impl SynthMode {
    pub fn voices_per_note(self) -> usize {
        match self {
            SynthMode::Mono => 8,
            SynthMode::BiPoly => 4,
            SynthMode::QuadPoly => 2,
            SynthMode::OctoPoly => 1,
        }
    }

    pub fn max_active_notes(self) -> usize {
        AMOUNT_OF_VOICES / self.voices_per_note()
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct SynthConfiguration {
    pub mode: SynthMode,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
    pub voices: [WaveformOscillatorConfig; AMOUNT_OF_VOICES],
    pub envelops: [EnvelopConfiguration; AMOUNT_OF_VOICES],
    pub lfos: [WaveformLFOConfig; AMOUNT_OF_VOICES / 2],
    pub filter_config: FilterConfig,
    pub mixer_config: MixerConfiguration,
    pub overdrive_config: OverdriveConfiguration,
    pub bitcrunch_config: BitcrunchConfiguration,
    pub delay_config: DelayConfiguration,
    pub routering_config: RoutingConfiguration,
    pub synth_config: SynthConfiguration,
    #[serde(default)]
    pub flanger_config: FlangerConfiguration,
    #[serde(default)]
    pub fm_config: FmConfiguration,
}
