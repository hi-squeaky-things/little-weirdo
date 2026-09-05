//! Flanger effect.
use crate::{effects::Effect, math};
use alloc::collections::VecDeque;
use serde::{Deserialize, Serialize};

extern crate alloc;

const MAX_DELAY_TIME_MS: u16 = 2_000;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct FlangerConfiguration {
    pub enabled: bool,
    pub base_delay_time_ms: u16,
    pub depth_ms: u16,
    pub rate_hz: u16,
    pub mix_percentage: u8,
}

impl Default for FlangerConfiguration {
    fn default() -> Self {
        Self {
            enabled: false,
            base_delay_time_ms: 1,
            depth_ms: 1,
            rate_hz: 1,
            mix_percentage: 50,
        }
    }
}

pub struct Flanger {
    pub config: FlangerConfiguration,
    buffer: VecDeque<i16>,
    sample_rate: u16,
    max_delay_samples: usize,
    modulation_period: usize,
    modulation_phase: usize,
}

impl Flanger {
    pub fn new(config: FlangerConfiguration, sample_rate: u16) -> Self {
        let mut flanger = Self {
            config,
            buffer: VecDeque::new(),
            sample_rate,
            max_delay_samples: 0,
            modulation_period: 1,
            modulation_phase: 0,
        };
        flanger.reload(config, sample_rate);
        flanger
    }

    pub fn reload(&mut self, config: FlangerConfiguration, sample_rate: u16) {
        self.config = config;
        self.sample_rate = sample_rate;
        self.max_delay_samples = self.maximum_delay_in_samples();
        self.modulation_period = self.modulation_period();
        self.modulation_phase = 0;
        self.buffer.clear();
        self.buffer
            .reserve(self.max_delay_samples.saturating_add(1));
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.modulation_phase = 0;
    }

    fn milliseconds_to_samples(&self, milliseconds: u16) -> usize {
        milliseconds.min(MAX_DELAY_TIME_MS) as usize * self.sample_rate as usize / 1_000
    }

    fn maximum_delay_in_samples(&self) -> usize {
        self.milliseconds_to_samples(
            self.config
                .base_delay_time_ms
                .saturating_add(self.config.depth_ms)
                .min(MAX_DELAY_TIME_MS),
        )
    }

    fn modulation_period(&self) -> usize {
        if self.config.rate_hz == 0 || self.sample_rate == 0 {
            1
        } else {
            (self.sample_rate as usize / self.config.rate_hz as usize).max(1)
        }
    }

    fn current_delay_in_samples(&self) -> usize {
        let base_delay = self.milliseconds_to_samples(self.config.base_delay_time_ms);
        let depth = self.milliseconds_to_samples(self.config.depth_ms);
        if depth == 0 || self.config.rate_hz == 0 {
            return base_delay.min(self.max_delay_samples);
        }

        let period = self.modulation_period;
        let triangle = if period <= 1 {
            100
        } else {
            let half_period = period / 2;
            if self.modulation_phase < half_period {
                self.modulation_phase * 200 / period
            } else {
                (period - self.modulation_phase) * 200 / period
            }
        };
        let offset = (triangle as i32 - 100) * depth as i32 / 100;
        ((base_delay as i32 + offset).max(0) as usize).min(self.max_delay_samples)
    }

    fn advance_modulation(&mut self) {
        self.modulation_phase = (self.modulation_phase + 1) % self.modulation_period;
    }
}

impl Effect for Flanger {
    fn clock(&mut self, sample: i16) -> i16 {
        if !self.config.enabled {
            return sample;
        }

        let delay = self.current_delay_in_samples();
        self.buffer.push_back(sample);

        let delayed = if self.buffer.len() > delay {
            self.buffer[self.buffer.len() - delay - 1]
        } else {
            0
        };

        while self.buffer.len() > self.max_delay_samples.saturating_add(1) {
            self.buffer.pop_front();
        }

        self.advance_modulation();
        let mix = self.config.mix_percentage.min(100) as i16;
        math::percentage(sample, 100 - mix).saturating_add(math::percentage(delayed, mix))
    }
}
