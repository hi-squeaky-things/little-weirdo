//! Delay/Echo effect
use crate::{effects::Effect, math};
use serde::{Deserialize, Serialize};
extern crate alloc;
use alloc::collections::VecDeque;

// Runtime settings for the delay effect: whether it is active, how long the delay is in
// milliseconds,
// how much of the delayed signal is mixed back in, and whether feedback is enabled.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct DelayConfiguration {
    pub enabled: bool,
    pub delay_time: u16,
    #[serde(default)]
    pub delay_decrease_percentage: u8,
    pub mix_percentage: u8,
    pub feedback: bool,
    pub feedback_percentage: u8,
}

// Delay effect state: stores the current signal buffer and the configured delay length.
pub struct Delay {
    pub config: DelayConfiguration,
    buffer: VecDeque<i16>,
    delay_time: usize,
    delay_cycle_remaining: usize,
    initial_cycle_completed: bool,
    current_feedback_percentage: u8,
}

impl Delay {
    pub fn new(config: DelayConfiguration, sample_rate: u16) -> Self {
        let delay_time = Self::delay_time_in_samples(config.delay_time, sample_rate);
        Delay {
            config,
            // Reserve enough room for the delay buffer to grow without frequent reallocations.
            buffer: VecDeque::with_capacity(delay_time * 2),
            delay_time,
            delay_cycle_remaining: 0,
            initial_cycle_completed: false,
            current_feedback_percentage: config.feedback_percentage,
        }
    }

    // Update the effect settings while keeping the same instance alive.
    pub fn reload(&mut self, config: DelayConfiguration, sample_rate: u16) {
        self.config = config;
        self.delay_time = Self::delay_time_in_samples(config.delay_time, sample_rate);
        self.delay_cycle_remaining = 0;
        self.initial_cycle_completed = false;
        self.current_feedback_percentage = config.feedback_percentage;
    }

    pub fn reset(&mut self) {
        if self.config.delay_decrease_percentage != 0 {
            //self.buffer.clear();
            self.delay_cycle_remaining = 0;
            self.initial_cycle_completed = false;
            self.current_feedback_percentage = self.config.feedback_percentage;
        }
    }

    fn delay_time_in_samples(delay_time: u16, sample_rate: u16) -> usize {
        delay_time as usize * sample_rate as usize / 1000
    }

    fn advance_delay_cycle(&mut self) {
        let decrease_percentage = self.config.delay_decrease_percentage.min(100) as usize;
        if decrease_percentage == 0 || self.delay_time == 0 {
            return;
        }

        if !self.initial_cycle_completed {
            self.initial_cycle_completed = true;
            self.delay_cycle_remaining = self.delay_time;
            return;
        }

        if self.delay_cycle_remaining > 1 {
            self.delay_cycle_remaining -= 1;
            return;
        }

        self.current_feedback_percentage = (self.current_feedback_percentage as usize
            * (100 - decrease_percentage)
            / 100) as u8;
        self.delay_cycle_remaining = self.delay_time;
    }
}

impl Effect for Delay {
    fn clock(&mut self, sample: i16) -> i16 {
        // Convert config values to the types needed for the signal math.
        let delay_time = self.delay_time;
        let mix_percentage = self.config.mix_percentage as i16;
        let feedback = self.config.feedback;
        let feedback_percentage = self.current_feedback_percentage as i16;

        if self.config.enabled {
            // Once the buffer has enough audio to represent the requested delay, mix in the
            // delayed sample and feed it back when configured.
            if self.buffer.len() > delay_time {
                let sample_with_delay = sample.saturating_add(math::percentage(
                    self.buffer.pop_front().unwrap(),
                    mix_percentage,
                ));
                if feedback {
                    // Feedback re-inserts part of the processed signal back into the buffer.
                    self.buffer
                        .push_back(math::percentage(sample_with_delay, feedback_percentage));
                } else {
                    // Without feedback, the delay taps only the incoming signal.
                    self.buffer.push_back(sample);
                }
                self.advance_delay_cycle();
                return sample_with_delay;
            } else {
                // Fill the buffer until the delay time is reached.
                self.buffer.push_back(sample);
            }
        }
        sample
    }
}
