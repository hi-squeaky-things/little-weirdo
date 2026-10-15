//! Envelope generator
//!
//! This module implements an ADSR (Attack, Decay, Sustain, Release) envelope generator
//! that produces control signals for audio synthesis. The envelope responds to gate
//! signals and follows the standard ADSR envelope shape.

use serde::{Deserialize, Serialize};

use super::Clockable;

/// Represents the different states an envelope can be in
#[derive(PartialEq)]
pub enum EnvelopeState {
    /// No active signal, idle state
    Idle,
    /// Attack phase - rising from 0% to 100%
    Attack,
    /// Decay phase - falling from 100% to sustain level
    Decay,
    /// Sustain phase - maintaining the sustain level
    Sustain,
    /// Release phase - falling from current level to 0%
    Release,
}

/// Configuration parameters for the envelope generator
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct EnvelopConfiguration {
    /// Time for the attack phase in milliseconds
    pub attack_time: u16,
    /// Time for the decay phase in milliseconds  
    pub decay_time: u16,
    /// Time for the release phase in milliseconds
    pub release_time: u16,
    /// Sustain level percentage (0-100)
    pub sustain_level: i16,
}

/// Envelope generator implementation
pub struct EnvelopeGenerator {
    // Configuration
    /// The envelope configuration settings
    pub configuration: EnvelopConfiguration,
    // Control
    /// Gate signal state (true = gate open, false = gate closed)
    gate: bool,
    // Runtime State
    /// Current state of the envelope
    state: EnvelopeState,
    /// Cumulative time ticks since last state transition
    cumulative_time_tick: u32,
    /// Sample rate used to convert phase durations to sample ticks
    sample_rate: u16,
    /// Current output level as percentage (0-100)
    current_output_level_percentage: i16,
    /// Level at which release phase starts
    release_level: i16,
    /// Total time ticks for current phase
    cumulative_time_ticks_for_one_phase: u32,
    /// Level to restart from when retriggering
    retrigger_level: i16,
}

impl Clockable for EnvelopeGenerator {
    // Clock the envelope, return the percentage (0..100%) of the envelope.
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        let mut output: u32 = 0;

        // Silence when we are in the IDLE state.
        if self.state == EnvelopeState::Idle {
            return output as i16;
        }
        match self.state {
            EnvelopeState::Attack => {
                if self.cumulative_time_tick >= self.cumulative_time_ticks_for_one_phase {
                    output = 100;
                    self.decay();
                } else {
                    let attack_range = 100 - self.retrigger_level as u32;
                    let progress = self.cumulative_time_tick as u64 * attack_range as u64
                        / self.cumulative_time_ticks_for_one_phase as u64;
                    output = self.retrigger_level as u32 + progress as u32;
                }
            }
            EnvelopeState::Decay => {
                if self.cumulative_time_tick >= self.cumulative_time_ticks_for_one_phase {
                    output = self.configuration.sustain_level as u32;
                    self.transistion_state(EnvelopeState::Sustain);
                } else {
                    let drop_amount = 100 - self.configuration.sustain_level as u32;
                    let current_drop = (drop_amount as u64 * self.cumulative_time_tick as u64)
                        / self.cumulative_time_ticks_for_one_phase as u64;
                    output = 100 - current_drop as u32;
                }
            }
            EnvelopeState::Sustain => {
                output = self.configuration.sustain_level as u32;
            }

            EnvelopeState::Release => {
                if self.cumulative_time_tick >= self.cumulative_time_ticks_for_one_phase {
                    output = 0;
                    self.transistion_state(EnvelopeState::Idle);
                    self.cumulative_time_ticks_for_one_phase = 0;
                } else {
                    let remaining =
                        self.cumulative_time_ticks_for_one_phase - self.cumulative_time_tick;
                    let scaled_level = self.release_level as u64 * remaining as u64;
                    let duration = self.cumulative_time_ticks_for_one_phase as u64;
                    output = ((scaled_level + duration - 1) / duration) as u32;
                }
            }
            EnvelopeState::Idle => {}
        }
        self.cumulative_time_tick += 1;

        //safeguard, make sure we never output greater then 100%
        if output > 100 {
            output = 100;
        }

        self.current_output_level_percentage = output as i16;
        self.current_output_level_percentage
    }
}

impl EnvelopeGenerator {
    /// Creates a new envelope generator with the given configuration and sample rate
    pub fn new(envelop: EnvelopConfiguration, sample_rate: u16) -> Self {
        Self {
            configuration: envelop,
            gate: false,
            state: EnvelopeState::Idle,
            cumulative_time_tick: 0,
            sample_rate,
            current_output_level_percentage: 0,
            release_level: 0,
            cumulative_time_ticks_for_one_phase: 0,
            retrigger_level: 0,
        }
    }

    /// Reloads the envelope configuration
    pub fn reload(&mut self, envelop: EnvelopConfiguration) {
        self.configuration = envelop;
        self.reset();
    }

    /// Updates the envelope configuration without interrupting its current phase.
    pub fn update_configuration(&mut self, envelop: EnvelopConfiguration) {
        let was_releasing = self.state == EnvelopeState::Release;
        self.configuration = envelop;
        if was_releasing {
            self.release_level = self.current_output_level_percentage;
            self.cumulative_time_tick = 0;
            self.cumulative_time_ticks_for_one_phase =
                self.duration_in_ticks(self.configuration.release_time as u32);
        }
    }

    fn duration_in_ticks(&self, duration_ms: u32) -> u32 {
        (duration_ms as u64 * self.sample_rate as u64 / 1000).min(u32::MAX as u64) as u32
    }

    /// Transitions the envelope to a new state and resets tick counter
    fn transistion_state(&mut self, state: EnvelopeState) {
        self.state = state;
        self.cumulative_time_tick = 0;
    }

    pub fn reset(&mut self) {
        self.gate = false;
        self.state = EnvelopeState::Idle;
        self.cumulative_time_tick = 0;
        self.current_output_level_percentage = 0;
        self.release_level = 0;
        self.cumulative_time_ticks_for_one_phase = 0;
        self.retrigger_level = 0;
    }

    /// Closes the gate signal, initiating release phase if configured
    pub fn close_gate(&mut self) {
        if !self.gate {
            return;
        };

        self.cumulative_time_tick = 0;
        self.gate = false;
        if self.configuration.release_time > 0 {
            self.state = EnvelopeState::Release;
            self.release_level = self.current_output_level_percentage;
            self.cumulative_time_ticks_for_one_phase =
                self.duration_in_ticks(self.configuration.release_time as u32);
        } else {
            self.state = EnvelopeState::Idle;
        }
    }

    /// Transition to decay phase from attack phase
    fn decay(&mut self) {
        self.transistion_state(EnvelopeState::Decay);
        self.cumulative_time_ticks_for_one_phase =
            self.duration_in_ticks(self.configuration.decay_time as u32);
    }

    /// Opens the gate signal, initiating attack phase
    pub fn open_gate(&mut self) {
        self.gate = true;
        self.transistion_state(EnvelopeState::Attack);
        self.retrigger_level = self.current_output_level_percentage.clamp(0, 100);
        let attack_range = 100 - self.retrigger_level as u32;
        if attack_range == 0 {
            self.decay();
            return;
        }

        let full_attack_ticks = self.duration_in_ticks(self.configuration.attack_time as u32);
        let adjusted_attack_ticks = (full_attack_ticks as u64 * attack_range as u64 / 100) as u32;
        self.cumulative_time_ticks_for_one_phase = if full_attack_ticks == 0 {
            0
        } else {
            adjusted_attack_ticks.max(1)
        };
    }
}
