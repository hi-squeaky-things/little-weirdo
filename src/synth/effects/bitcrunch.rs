use core::i16;

use serde::{Deserialize, Serialize};

use crate::synth::effects::Effect;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct BitcrunchConfiguration {
    pub enabled: bool,
}

pub struct Bitcrunch {
    pub config: BitcrunchConfiguration,
}

impl Bitcrunch {
    pub fn new(config: BitcrunchConfiguration) -> Self {
        Bitcrunch { config }
    }
}

impl Effect for Bitcrunch {
    fn clock(&mut self, sample: i16) -> i16 {
        // 8-bit style: reduce to 8-bit range (0-255) then scale back to i16 range
        if self.config.enabled {
            return (sample / 256) * 256;
        }
        sample
    }
}
