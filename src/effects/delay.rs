use core::i16;

use serde::{Deserialize, Serialize};

use crate::{effects::Effect, math};
extern crate alloc;

use alloc::collections::VecDeque;



#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct DelayConfiguration {
    pub enabled: bool,
    pub delay_time: u16,
    pub mix_percentage: u8,
    pub feedback: bool,
    pub feedback_percentage: u8
}

pub struct Delay {
    pub config: DelayConfiguration,
    buffer: VecDeque<i16>,
    delay_time: u16,
}

impl Delay {
    pub fn new(config: DelayConfiguration) -> Self {
        Delay {
            config,
            buffer: VecDeque::with_capacity((config.delay_time * 2) as usize),
            delay_time: config.delay_time,
        }
    }
}

impl Effect for Delay {
    fn clock(&mut self, sample: i16) -> i16 {
        if self.config.enabled {
            if self.buffer.len() > self.delay_time as usize {
                let sample_with_delay = sample
                    + math::percentage(self.buffer.pop_front().unwrap(), self.config.mix_percentage as i16);
                if self.config.feedback {
                    self.buffer.push_back( math::percentage(sample_with_delay,  self.config.feedback_percentage as i16));
                } else {
                    self.buffer.push_back(sample);
                }
                return sample_with_delay;
            } else {
                self.buffer.push_back(sample);
            }
        }
        sample
    }
}
