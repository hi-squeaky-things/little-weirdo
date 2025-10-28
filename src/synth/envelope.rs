//! Envelope generator

use serde::{Deserialize, Serialize};

use crate::synth::math::percentage;

use super::Clockable;

#[derive(PartialEq, Clone, Copy)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct EnvelopConfiguration {
    pub attack_time: i16,
    pub decay_time: i16,
    pub release_time: i16,
    pub sustain_level: i16,
}

pub struct EnvelopeGenerator {
    // Configuration
    pub configuration: EnvelopConfiguration,
    // Control
    gate: bool,
    // Runtime State
    state: EnvelopeState,
    cumulative_time_tick: u32,
    time_ticks_per_ms: u16,
    ticks_per_percent: u32,
    current_output_level_percentage: i16,
    release_level: i16,
    phase_ticks: u32,
    retrigger_level: i16,
    gate_open_counter: u8,
}

impl Clockable for EnvelopeGenerator {
    // Clock the envelope, return the percentage (0..100%) of the envelope.
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        // Silence when we are in the IDLE state and gate is closed
        if !self.gate && self.state == EnvelopeState::Idle {
            return 0;
        }
        
        let output = match self.state {
            EnvelopeState::Attack => {
                if self.cumulative_time_tick >= self.phase_ticks {
                    self.decay();
                    100
                } else if self.ticks_per_percent > 0 {
                    self.retrigger_level as u32 + 
                        self.cumulative_time_tick / self.ticks_per_percent
                } else {
                    self.current_output_level_percentage as u32
                }
            }
            EnvelopeState::Decay => {
                if self.cumulative_time_tick >= self.phase_ticks {
                    self.transistion_state(EnvelopeState::Sustain);
                    self.configuration.sustain_level as u32
                } else {
                    let progress = self.cumulative_time_tick / self.ticks_per_percent;
                    100 - percentage(100 - self.configuration.sustain_level, progress as i16) as u32
                }
            }
            EnvelopeState::Sustain => self.configuration.sustain_level as u32,
            EnvelopeState::Release => {
                if self.cumulative_time_tick >= self.phase_ticks {
                    self.transistion_state(EnvelopeState::Idle);
                    0
                } else {
                    let progress = self.cumulative_time_tick / self.ticks_per_percent;
                    if progress < 100 {
                        percentage(self.release_level, (100 - progress) as i16) as u32
                    } else {
                        0
                    }
                }
            }
            EnvelopeState::Idle => 0,
        };
        
        self.cumulative_time_tick += 1;
        
        // Clamp output to 0-100 range
        output.min(100) as i16
    }
}

impl EnvelopeGenerator {
    pub fn new(envelop: EnvelopConfiguration, sample_rate: u16) -> Self {
        let mut envelope = Self {
            configuration: envelop,
            gate: false,
            state: EnvelopeState::Idle,
            cumulative_time_tick: 0,
            time_ticks_per_ms: sample_rate / 1000,
            ticks_per_percent: 0,
            current_output_level_percentage: 0,
            release_level: 0,
            phase_ticks: 0,
            retrigger_level: 0,
            gate_open_counter: 0,
        };
        
        // Pre-calculate common values
        envelope.update_phase_ticks();
        envelope
    }

    pub fn reload(&mut self, envelop: EnvelopConfiguration) {
        self.configuration = envelop;
        self.update_phase_ticks();
    }

    fn update_phase_ticks(&mut self) {
        // This is called during initialization and reload
        // We don't recalculate ticks_per_percent here since it's calculated per phase
    }

    fn transistion_state(&mut self, state: EnvelopeState) {
        self.state = state;
        self.cumulative_time_tick = 0;
    }

    pub fn close_gate(&mut self) {
        if !self.gate {
            return;
        }
        
        self.gate_open_counter = self.gate_open_counter.saturating_sub(1);
        
        if self.gate_open_counter == 0 {
            self.cumulative_time_tick = 0;
            self.gate = false;
            
            if self.configuration.release_time > 0 {
                self.state = EnvelopeState::Release;
                self.release_level = if self.release_level != self.configuration.sustain_level {
                    self.current_output_level_percentage
                } else {
                    self.configuration.sustain_level
                };
                self.ticks_per_percent = (self.configuration.release_time as u32 * self.time_ticks_per_ms as u32) / 100;
                self.phase_ticks = self.configuration.release_time as u32 * self.time_ticks_per_ms as u32;
            } else {
                self.state = EnvelopeState::Idle;
            }
        }
    }

    fn decay(&mut self) {
        self.transistion_state(EnvelopeState::Decay);
        self.ticks_per_percent = (self.configuration.decay_time as u32 * self.time_ticks_per_ms as u32) / 100;
        self.phase_ticks = self.configuration.decay_time as u32 * self.time_ticks_per_ms as u32;
    }

    pub fn open_gate(&mut self) {
        self.gate = true;
        self.cumulative_time_tick = 0;
        self.gate_open_counter += 1;
        self.transistion_state(EnvelopeState::Attack);
        
        let mut adjusted_attack_time = self.configuration.attack_time as u32;
        self.retrigger_level = 0;
        
        if self.current_output_level_percentage > 0 {
            // Calculate the remaining slope time to reach 100%
            let percentage_remaining = 100 - self.current_output_level_percentage;
            if percentage_remaining > 0 {
                self.retrigger_level = self.current_output_level_percentage;
                adjusted_attack_time = percentage(self.configuration.attack_time, percentage_remaining) as u32;
                self.ticks_per_percent = (adjusted_attack_time * self.time_ticks_per_ms as u32) / percentage_remaining as u32;
            } else {
                self.decay();
                return;
            }
        } else {
            self.ticks_per_percent = (adjusted_attack_time * self.time_ticks_per_ms as u32) / 100;
        }
        
        self.phase_ticks = adjusted_attack_time * self.time_ticks_per_ms as u32;
    }
}