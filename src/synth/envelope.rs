//! Envelope generator
//!
//! This module implements an ADSR (Attack, Decay, Sustain, Release) envelope generator
//! that produces control signals for audio synthesis. The envelope responds to gate
//! signals and follows the standard ADSR envelope shape.

use serde::{Deserialize, Serialize};

use super::Clockable;

const LEVEL_SCALE: u32 = 1 << 16;

fn level_percentage(level: u32) -> u32 {
    (level as u64 * 100 / LEVEL_SCALE as u64) as u32
}

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
    /// Current output level, where 0..LEVEL_SCALE represents 0..100%.
    current_output_level: u32,
    /// Level at which release phase starts
    release_level: u32,
    /// Total time ticks for current phase
    cumulative_time_ticks_for_one_phase: u32,
    /// Level to restart from when retriggering
    retrigger_level: u32,
}

impl Clockable for EnvelopeGenerator {
    // Clock the envelope, return the percentage (0..100%) of the envelope.
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        let level = self.clock_fixed();
        let percentage = level_percentage(level) as i16;
        if level > 0 {
            percentage.max(1)
        } else {
            percentage
        }
    }
}

impl EnvelopeGenerator {
    /// Advances the envelope and returns its normalized Q16 level (0..65,536).
    pub(crate) fn clock_fixed(&mut self) -> u32 {
        let mut output = 0;

        if self.state == EnvelopeState::Idle {
            return 0;
        }
        match self.state {
            EnvelopeState::Attack => {
                if self.cumulative_time_tick >= self.cumulative_time_ticks_for_one_phase {
                    output = LEVEL_SCALE;
                    self.decay();
                } else {
                    let attack_range = LEVEL_SCALE - self.retrigger_level;
                    let progress = self.cumulative_time_tick as u64 * attack_range as u64
                        / self.cumulative_time_ticks_for_one_phase as u64;
                    output = self.retrigger_level + progress as u32;
                }
            }
            EnvelopeState::Decay => {
                if self.cumulative_time_tick >= self.cumulative_time_ticks_for_one_phase {
                    output = self.sustain_level_fixed();
                    self.transistion_state(EnvelopeState::Sustain);
                } else {
                    let sustain_level = self.sustain_level_fixed();
                    let drop_amount = LEVEL_SCALE - sustain_level;
                    let current_drop = (drop_amount as u64 * self.cumulative_time_tick as u64)
                        / self.cumulative_time_ticks_for_one_phase as u64;
                    output = LEVEL_SCALE - current_drop as u32;
                }
            }
            EnvelopeState::Sustain => {
                output = self.sustain_level_fixed();
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
                    output = scaled_level.div_ceil(duration) as u32;
                }
            }
            EnvelopeState::Idle => {}
        }
        self.cumulative_time_tick += 1;

        self.current_output_level = output.min(LEVEL_SCALE);
        self.current_output_level
    }

    fn sustain_level_fixed(&self) -> u32 {
        (self.configuration.sustain_level.clamp(0, 100) as u32 * LEVEL_SCALE) / 100
    }

    /// Creates a new envelope generator with the given configuration and sample rate
    pub fn new(envelop: EnvelopConfiguration, sample_rate: u16) -> Self {
        Self {
            configuration: envelop,
            gate: false,
            state: EnvelopeState::Idle,
            cumulative_time_tick: 0,
            sample_rate,
            current_output_level: 0,
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
            self.release_level = self.current_output_level;
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
        self.current_output_level = 0;
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
            self.release_level = self.current_output_level;
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
        self.retrigger_level = self.current_output_level.min(LEVEL_SCALE);
        if self.retrigger_level == LEVEL_SCALE {
            self.decay();
            return;
        }

        let current_level_percentage = level_percentage(self.retrigger_level);
        let attack_range_percentage = 100 - current_level_percentage;
        let full_attack_ticks = self.duration_in_ticks(self.configuration.attack_time as u32);
        let adjusted_attack_ticks =
            (full_attack_ticks as u64 * attack_range_percentage as u64 / 100) as u32;
        self.cumulative_time_ticks_for_one_phase = if full_attack_ticks == 0 {
            0
        } else {
            adjusted_attack_ticks.max(1)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{EnvelopConfiguration, EnvelopeGenerator, LEVEL_SCALE};
    use crate::synth::Clockable;

    #[test]
    fn long_release_advances_smoothly_until_its_exact_endpoint() {
        let mut envelope = EnvelopeGenerator::new(
            EnvelopConfiguration {
                attack_time: 0,
                decay_time: 0,
                release_time: 60_000,
                sustain_level: 100,
            },
            1_000,
        );
        envelope.open_gate();
        envelope.clock_fixed();
        envelope.clock_fixed();
        envelope.close_gate();

        let mut previous_level = envelope.clock_fixed();
        let mut changed_samples = 0;
        for _ in 1..60_000 {
            let level = envelope.clock_fixed();
            assert!(level <= previous_level);
            assert!(previous_level - level <= 2);
            assert!(level > 0);
            changed_samples += usize::from(level != previous_level);
            previous_level = level;
        }

        assert!(changed_samples > 50_000);
        assert_eq!(envelope.clock_fixed(), 0);
    }

    #[test]
    fn public_clock_keeps_integer_release_projection() {
        let mut envelope = EnvelopeGenerator::new(
            EnvelopConfiguration {
                attack_time: 0,
                decay_time: 0,
                release_time: 10,
                sustain_level: 100,
            },
            1_000,
        );
        envelope.open_gate();
        envelope.clock(None);
        envelope.clock(None);
        envelope.close_gate();

        for expected in (1..=10).rev().map(|step| step * 10) {
            assert_eq!(envelope.clock(None), expected);
        }
        assert_eq!(envelope.clock(None), 0);
    }

    #[test]
    fn fractional_level_survives_release_update_and_retrigger() {
        let mut envelope = EnvelopeGenerator::new(
            EnvelopConfiguration {
                attack_time: 0,
                decay_time: 0,
                release_time: 60_000,
                sustain_level: 100,
            },
            1_000,
        );
        envelope.open_gate();
        envelope.clock_fixed();
        envelope.clock_fixed();
        envelope.close_gate();
        envelope.clock_fixed();
        let fractional_level = envelope.clock_fixed();
        assert!(fractional_level < LEVEL_SCALE);

        envelope.update_configuration(EnvelopConfiguration {
            attack_time: 100,
            decay_time: 0,
            release_time: 50_000,
            sustain_level: 100,
        });
        assert_eq!(envelope.release_level, fractional_level);
        assert_eq!(envelope.clock_fixed(), fractional_level);

        envelope.open_gate();
        assert_eq!(envelope.retrigger_level, fractional_level);
        assert_eq!(envelope.clock_fixed(), fractional_level);
    }
}
