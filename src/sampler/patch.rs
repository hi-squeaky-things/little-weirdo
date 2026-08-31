//! Patch to config the sounds
extern crate alloc;
use alloc::{string::String, vec::Vec};
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
    
    /// SoundFont-like zones: different samples for different note ranges
    #[serde(default)]
    pub zones: Vec<Zone>,
    
   
}

/// SoundFont-like zone definition
#[derive(Clone, Serialize, Deserialize)]
pub struct Zone {
    /// Start note of the zone (inclusive)
    pub start_note: u8,
    
    /// End note of the zone (inclusive)
    pub end_note: u8,
    
    /// Sample map for this zone
    pub sample_map: u8,
    
    /// Base key for this zone
    pub base_key: u8,
    
    /// Loop start for this zone
    pub loop_start: u32,
    
    /// Loop end for this zone
    pub loop_end: u32,
    
    /// One shot for this zone
    pub one_shot: bool,
}
