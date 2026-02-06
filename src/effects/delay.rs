use core::i16;

use serde::{Deserialize, Serialize};

use crate::{effects::Effect, math};
extern crate alloc;

use alloc::collections::VecDeque;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct DelayConfiguration {
    pub enabled: bool,
    pub delay_time: u16,
    pub mix: u8,
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
        // 8-bit style: reduce to 8-bit range (0-255) then scale back to i16 range
        if self.config.enabled {
            self.buffer.push_back(sample);
            if self.buffer.len() > self.delay_time as usize {
                let sample_with_delay = sample +  math::percentage(self.buffer.pop_front().unwrap(), self.config.mix as i16);
                return sample_with_delay;
            }
        }

        sample
    }
}
