//! Patch to config the sounds
extern crate alloc;
use alloc::{string::String};
use serde::{Deserialize, Serialize};

use crate::{
    effects::{
        bitcrunch::BitcrunchConfiguration, delay::DelayConfiguration,
        flanger::FlangerConfiguration, overdrive::OverdriveConfiguration,
    }, synth::envelope::EnvelopConfiguration, wavetable::sample_voice::SampleVoiceConfig,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
    pub sample_config: SampleVoiceConfig,
    pub overdrive_config: OverdriveConfiguration,
    pub bitcrunch_config: BitcrunchConfiguration,
    pub delay_config: DelayConfiguration,
    pub env_config: EnvelopConfiguration,
    #[serde(default)]
    pub flanger_config: FlangerConfiguration,
}
