use core::i16;

use serde::{Deserialize, Serialize};

use crate::{effects::Effect, math};
extern crate alloc;

use alloc::collections::VecDeque;

// Runtime settings for the delay effect: whether it is active, how long the delay is,
// how much of the delayed signal is mixed back in, and whether feedback is enabled.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct DelayConfiguration {
    pub enabled: bool,
    pub delay_time: u16,
    pub mix_percentage: u8,
    pub feedback: bool,
    pub feedback_percentage: u8,
}

// Delay effect state: stores the current signal buffer and the configured delay length.
pub struct Delay {
    pub config: DelayConfiguration,
    buffer: VecDeque<i16>,
    delay_time: u16,
}

impl Delay {
    pub fn new(config: DelayConfiguration) -> Self {
        Delay {
            config,
            // Reserve enough room for the delay buffer to grow without frequent reallocations.
            buffer: VecDeque::with_capacity((config.delay_time * 2) as usize),
            delay_time: config.delay_time,
        }
    }

    // Update the effect settings while keeping the same instance alive.
    pub fn reload(&mut self, config: DelayConfiguration) {
        self.config = config;
        self.delay_time = config.delay_time;
    }
}

impl Effect for Delay {
    fn clock(&mut self, sample: i16) -> i16 {
        // Convert config values to the types needed for the signal math.
        let delay_time = self.delay_time as usize;
        let mix_percentage = self.config.mix_percentage as i16;
        let feedback = self.config.feedback;
        let feedback_percentage = self.config.feedback_percentage as i16;

        if self.config.enabled {
            // Once the buffer has enough audio to represent the requested delay, mix in the
            // delayed sample and feed it back when configured.
            if self.buffer.len() > delay_time {
                let sample_with_delay = sample
                    + math::percentage(self.buffer.pop_front().unwrap(), mix_percentage);
                if feedback {
                    // Feedback re-inserts part of the processed signal back into the buffer.
                    self.buffer.push_back(math::percentage(sample_with_delay, feedback_percentage));
                } else {
                    // Without feedback, the delay taps only the incoming signal.
                    self.buffer.push_back(sample);
                }
                return sample_with_delay;
            } else {
                // Fill the buffer until the delay time is reached.
                self.buffer.push_back(sample);
            }
        }
        sample
    }
}
