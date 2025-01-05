//! Routing the signals

use super::AMOUNT_OF_VOICES;

#[derive(Copy, Clone)]
pub struct VoiceToEnvelopRoute {
    pub env: u8,
}
#[derive(Copy, Clone)]
pub struct VoiceToLFORoute {
    pub voices: [u8;2],
    pub enable: bool, 
}

#[derive(Copy, Clone)]
pub struct RoutingConfiguration {
    pub voices_to_envelop: [VoiceToEnvelopRoute;AMOUNT_OF_VOICES],
    pub voice_to_lfo: [VoiceToLFORoute; AMOUNT_OF_VOICES/2],
    pub lfo_to_filter: bool,
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