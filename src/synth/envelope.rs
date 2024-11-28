//! Envelope generator

use crate::synth::math::percentage;

use super::math;

#[derive(PartialEq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Envelop {
    pub attack_time: i16,
    pub decay_time: i16,
    pub release_time: i16,
    pub sustain_level: i16,
}

pub struct EnvelopeGenerator {
    // Configuration
    pub envelop: Envelop,
    // Control
    gate: bool,
    // Runtime State
    state: EnvelopeState,
    t: u32,
    clock_counter_1ms: u16,
    envelop_divider: u32,
    current_level: i16,
    release_level: i16,
    cycle_end_time: u32,
    retrigger_level: i16,
    gate_open_counter: u8,
}

impl EnvelopeGenerator {
    pub fn new(envelop: Envelop, sample_rate: u16) -> Self {
        Self {
            envelop,
            gate: false,
            state: EnvelopeState::Idle,
            t: 0,
            clock_counter_1ms: sample_rate / 1000,
            envelop_divider: 0,
            current_level: 0,
            release_level: 0,
            cycle_end_time: 0,
            retrigger_level: 0,
            gate_open_counter: 0,
        }
    }

    pub fn clock(&mut self) -> i16 {
        if !self.gate && self.state == EnvelopeState::Idle {
            return 0;
        }
        let mut output = 0;
        match self.state {
            EnvelopeState::Attack => {
                if self.t > self.cycle_end_time {
                    output = 100;
                    self.decay();
                } else {
                    if self.envelop_divider > 0 {
                        output = self.retrigger_level as u32 + self.t / self.envelop_divider;
                    } else {
                        output = self.current_level as u32;
                    }
                }
            }
            EnvelopeState::Decay => {
                if self.t > self.cycle_end_time {
                    output = self.envelop.sustain_level as u32;
                    self.transistion_state(EnvelopeState::Sustain);
                } else {
                    output = 100
                        - math::percentage(
                            100 - self.envelop.sustain_level as i16,
                            (self.t / self.envelop_divider) as i16,
                        ) as u32;
                }
            }
            EnvelopeState::Sustain => {
                output = self.envelop.sustain_level as u32;
            }
            EnvelopeState::Release => {
                if self.t > self.cycle_end_time {
                    output = 0;
                    self.transistion_state(EnvelopeState::Idle);
                    self.cycle_end_time = 0;
                } else {
                    //TODO: weird fix. need beter investigation, probably some rounding issue.
                    if self.t / self.envelop_divider < 101 {
                        output = math::percentage(
                            self.release_level as i16,
                            (100 - self.t / self.envelop_divider) as i16,
                        ) as u32;
                    } else {
                        output = 0;
                    }
                }
            }
            EnvelopeState::Idle => {}
        }
        self.t += 1;
        self.current_level = output as i16;
        self.current_level
    }

    fn transistion_state(&mut self, state: EnvelopeState) {
        self.state = state;
        self.t = 0;
    }

    pub fn close_gate(&mut self) {
        if !self.gate {
            return;
        };
        if self.gate_open_counter > 0 {
            self.gate_open_counter -= 1;
        }

        if self.gate_open_counter == 0 {
            self.t = 0;
            self.gate = false;
            if self.envelop.release_time > 0 {
                self.state = EnvelopeState::Release;
                self.release_level = if self.release_level != self.envelop.sustain_level {
                    self.current_level
                } else {
                    self.envelop.sustain_level
                };
                self.envelop_divider =
                    (self.envelop.release_time as u32 * self.clock_counter_1ms as u32) / 100;
                self.cycle_end_time =
                    self.envelop.release_time as u32 * self.clock_counter_1ms as u32;
            } else {
                self.state = EnvelopeState::Idle;
            }
        }
    }

    fn decay(&mut self) {
        self.transistion_state(EnvelopeState::Decay);
        self.envelop_divider =
            (self.envelop.decay_time as u32 * self.clock_counter_1ms as u32) / 100;
        self.cycle_end_time = self.envelop.decay_time as u32 * self.clock_counter_1ms as u32;
    }

    pub fn open_gate(&mut self) {
        self.gate = true;
        self.t = 0;
        self.gate_open_counter += 1;
        self.transistion_state(EnvelopeState::Attack);
        let mut adjusted_attack_time = self.envelop.attack_time as u32;
        self.retrigger_level = 0;
        if self.current_level > 0 {
            // calculate the remaining slope time to reach 100%
            let percentage_remaining = 100 - self.current_level;
            if percentage_remaining > 0 {
                self.retrigger_level = self.current_level;
                adjusted_attack_time =
                    percentage(self.envelop.attack_time, percentage_remaining) as u32;
                self.envelop_divider = (adjusted_attack_time as u32
                    * self.clock_counter_1ms as u32)
                    / percentage_remaining as u32;
            } else {
                self.decay();
            }
        } else {
            self.envelop_divider =
                (adjusted_attack_time as u32 * self.clock_counter_1ms as u32) / 100;
        }
        self.cycle_end_time = adjusted_attack_time as u32 * self.clock_counter_1ms as u32;
    }
}
