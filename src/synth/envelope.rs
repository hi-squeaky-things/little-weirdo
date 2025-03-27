//! Envelope generator

use serde::{Deserialize, Serialize};

use crate::synth::math::percentage;

use super::{math, Clockable};

#[derive(PartialEq)]
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
    amount_of_time_ticks_for_1ms: u16,
    amout_of_time_ticks_needed_for_one_percent_increase: u32,
    current_output_level_percentage: i16,
    release_level: i16,
    cumalative_time_ticks_for_one_phase: u32,
    retrigger_level: i16,
    gate_open_counter: u8,
}

impl Clockable for EnvelopeGenerator {
    // Clock the evenlope, return the percentage (0..100%) of the envelope.
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        let mut output:u32 = 0;
    
        // Silence when we are in the IDLE state.
        if !self.gate && self.state == EnvelopeState::Idle {
            return output as i16;
        }
        match self.state {
            EnvelopeState::Attack => {
                if self.cumulative_time_tick > self.cumalative_time_ticks_for_one_phase {
                    output = 100;
                    self.decay();
                } else {
                    if self.amout_of_time_ticks_needed_for_one_percent_increase > 0 {
                        output = self.retrigger_level as u32 + self.cumulative_time_tick / self.amout_of_time_ticks_needed_for_one_percent_increase;
                    } else {
                        output = self.current_output_level_percentage as u32;
                    }
                }
            }
            EnvelopeState::Decay => {
                if self.cumulative_time_tick > self.cumalative_time_ticks_for_one_phase {
                    output = self.configuration.sustain_level as u32;
                    self.transistion_state(EnvelopeState::Sustain);
                } else {
                    output = 100
                        - math::percentage(
                            100 - self.configuration.sustain_level as i16,
                            (self.cumulative_time_tick / self.amout_of_time_ticks_needed_for_one_percent_increase) as i16,
                        ) as u32;
                }
            }
            EnvelopeState::Sustain => {
                output = self.configuration.sustain_level as u32;
            }
            EnvelopeState::Release => {
                if self.cumulative_time_tick > self.cumalative_time_ticks_for_one_phase {
                    output = 0;
                    self.transistion_state(EnvelopeState::Idle);
                    self.cumalative_time_ticks_for_one_phase = 0;
                } else {
                    //TODO: weird fix. need beter investigation, probably some rounding issue.
                    if self.cumulative_time_tick / self.amout_of_time_ticks_needed_for_one_percent_increase < 101 {
                        output = math::percentage(
                            self.release_level as i16,
                            (100 - self.cumulative_time_tick / self.amout_of_time_ticks_needed_for_one_percent_increase) as i16,
                        ) as u32;
                    } else {
                        output = 0;
                    }
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
    pub fn new(envelop: EnvelopConfiguration, sample_rate: u16) -> Self {
        Self {
            configuration: envelop,
            gate: false,
            state: EnvelopeState::Idle,
            cumulative_time_tick: 0,
            amount_of_time_ticks_for_1ms: sample_rate / 1000,
            amout_of_time_ticks_needed_for_one_percent_increase: 0,
            current_output_level_percentage: 0,
            release_level: 0,
            cumalative_time_ticks_for_one_phase: 0,
            retrigger_level: 0,
            gate_open_counter: 0,
        }
    }

    pub fn reload(&mut self, envelop: EnvelopConfiguration) {
        self.configuration = envelop;
    }

    fn transistion_state(&mut self, state: EnvelopeState) {
        self.state = state;
        self.cumulative_time_tick = 0;
    }

    pub fn close_gate(&mut self) {
        if !self.gate {
            return;
        };
        if self.gate_open_counter > 0 {
            self.gate_open_counter -= 1;
        }

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
                self.amout_of_time_ticks_needed_for_one_percent_increase =
                    (self.configuration.release_time as u32 * self.amount_of_time_ticks_for_1ms as u32) / 100;
                self.cumalative_time_ticks_for_one_phase =
                    self.configuration.release_time as u32 * self.amount_of_time_ticks_for_1ms as u32;
            } else {
                self.state = EnvelopeState::Idle;
            }
        }
    }

    fn decay(&mut self) {
        self.transistion_state(EnvelopeState::Decay);
        self.amout_of_time_ticks_needed_for_one_percent_increase =
            (self.configuration.decay_time as u32 * self.amount_of_time_ticks_for_1ms as u32) / 100;
        self.cumalative_time_ticks_for_one_phase = self.configuration.decay_time as u32 * self.amount_of_time_ticks_for_1ms as u32;
    }

    pub fn open_gate(&mut self) {
        self.gate = true;
        self.cumulative_time_tick = 0;
        self.gate_open_counter += 1;
        self.transistion_state(EnvelopeState::Attack);
        let mut adjusted_attack_time = self.configuration.attack_time as u32;
        self.retrigger_level = 0;
        if self.current_output_level_percentage > 0 {
            // calculate the remaining slope time to reach 100%
            let percentage_remaining = 100 - self.current_output_level_percentage;
            if percentage_remaining > 0 {
                self.retrigger_level = self.current_output_level_percentage;
                adjusted_attack_time =
                    percentage(self.configuration.attack_time, percentage_remaining) as u32;
                self.amout_of_time_ticks_needed_for_one_percent_increase = (adjusted_attack_time as u32
                    * self.amount_of_time_ticks_for_1ms as u32)
                    / percentage_remaining as u32;
            } else {
                self.decay();
            }
        } else {
            self.amout_of_time_ticks_needed_for_one_percent_increase =
                (adjusted_attack_time as u32 * self.amount_of_time_ticks_for_1ms as u32) / 100;
        }
        self.cumalative_time_ticks_for_one_phase = adjusted_attack_time as u32 * self.amount_of_time_ticks_for_1ms as u32;
    }
}
