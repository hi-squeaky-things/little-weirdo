//! Patch to config the sounds
extern crate alloc;
use alloc::string::String;
use serde::{Deserialize, Serialize};

use crate::effects::{bitcrunch::BitcrunchConfiguration, delay::DelayConfiguration, filter::FilterConfig, overdrive::OverdriveConfiguration};


#[derive(Clone, Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
  //  pub filter_config: FilterConfig,
    pub overdrive_config: OverdriveConfiguration,
    pub bitcrunch_config: BitcrunchConfiguration,
    pub delay_config: DelayConfiguration,
}
