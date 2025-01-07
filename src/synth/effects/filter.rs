//! Lowpass filter
use serde::{Deserialize, Serialize};

use crate::synth::effects::Effect;
use crate::synth::math::{fx_mul, ifx_mul, ucfx_mul};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum KindOfFilter {
    Low,
    High,
    Band,
    Notch,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub cutoff_frequency: u16,
    pub resonance: u16,
    pub kind_of_filter: KindOfFilter,
    pub enabled: bool,
}

pub struct Filter {
    pub config: FilterConfig,
    buf0: i64,
    buf1: i64,
    feedback: u32,
}

impl Effect for Filter {
    fn clock(&mut self, sample: i16) -> i16 {
        if self.config.enabled {
            self.filter(sample)
        } else {
            sample
        }
    }
}

impl Filter {
    const SHIFTED_1: u16 = u16::MAX;
    pub fn new(config: FilterConfig) -> Self {
        let mut filter = Self {
            config: config,
            buf0: 0,
            buf1: 0,
            feedback: 0,
        };
        filter.prepare_filter();
        filter
    }

    pub fn reload(&mut self, config: FilterConfig) {
        self.config = config;
        self.prepare_filter();
    }

    fn prepare_filter(&mut self) {
        self.feedback = self.config.resonance as u32
            + ucfx_mul(
                self.config.resonance,
                Filter::SHIFTED_1 - self.config.cutoff_frequency,
            );
    }

    fn filter(&mut self, sample: i16) -> i16 {
        let highpass = sample as i64 - self.buf0;
        let bandpass = (self.buf0 - self.buf1) as i32;
       
        let feedback_on_bandpass = fx_mul(self.feedback as i64, bandpass);
        self.buf0 = self.buf0 + fx_mul(highpass + feedback_on_bandpass, self.config.cutoff_frequency as i32);
        self.buf1 = self.buf1 + ifx_mul(bandpass, self.config.cutoff_frequency) as i64;
        let out: i16;
        match self.config.kind_of_filter {
            KindOfFilter::Low => out = self.buf1 as i16,
            KindOfFilter::High => out = highpass as i16,
            KindOfFilter::Band => out = bandpass as i16,
            KindOfFilter::Notch => out = (sample as i64 - self.buf0 + self.buf1) as i16,
        }
        return out;
    }
}
