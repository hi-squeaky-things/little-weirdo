//! Routing the signals

use super::AMOUNT_OF_VOICE;

#[derive(Copy, Clone)]
pub struct VoiceToEnvelopRoute {
    pub env: u8,
}
#[derive(Copy, Clone)]
pub struct VoiceToLFORoute {
    pub voice: u8,
    pub enable: bool, 
}

#[derive(Copy, Clone)]
pub struct RoutingConfiguration {
    pub voices_to_envelop: [VoiceToEnvelopRoute;AMOUNT_OF_VOICE],
    pub voice_to_lfo: VoiceToLFORoute,
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