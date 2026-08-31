//! Patch to config the sounds
extern crate alloc;
use alloc::string::String;
use serde::{Deserialize, Serialize};

use crate::{effects::{bitcrunch::BitcrunchConfiguration, delay::DelayConfiguration, overdrive::OverdriveConfiguration}, synth::envelope::EnvelopConfiguration};


#[derive(Clone, Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
    pub drums: bool,
    pub sample_map: u8,
    pub loop_start: u32,
    pub loop_end: u32,
    pub one_shot: bool,
    pub base_key: u8,
    pub overdrive_config: OverdriveConfiguration,
    pub bitcrunch_config: BitcrunchConfiguration,
    pub delay_config: DelayConfiguration,
    pub env_config: EnvelopConfiguration,
}
