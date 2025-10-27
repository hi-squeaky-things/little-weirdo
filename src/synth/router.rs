//! Routing the signals

use serde::{Deserialize, Serialize};

use super::AMOUNT_OF_VOICES;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct VoiceToLFORoute {
    pub voices: [u8;2],
    pub enable: bool, 
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct RoutingConfiguration {
    pub voices_to_envelop: [u8;AMOUNT_OF_VOICES],
    pub lfo_to_voice: [VoiceToLFORoute; AMOUNT_OF_VOICES/2],
    pub lfo_to_filter: bool,
    pub lfo_to_freq: bool,
    pub lfo_to_freq_amount: u16,
}

pub struct Router {
    pub config: RoutingConfiguration,
}


impl Router {
    pub fn new(config: RoutingConfiguration) -> Self {
        Self {
            config
        }
    }

    pub fn reload(&mut self, config: RoutingConfiguration) {
        self.config = config;
    }
}