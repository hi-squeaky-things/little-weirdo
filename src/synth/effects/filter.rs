//! Lowpass filter
use crate::synth::effects::Effect;
use crate::synth::math::{self, fx_mul, ifx_mul, ucfx_mul};

#[derive(Copy, Clone)]
pub enum KindOfFilter {
    Low,
    High,
    Band,
    Notch,
}

#[derive(Copy, Clone)]
pub struct FilterConfig {
    pub cutoff_frequency: u16,
    pub resonance: u16,
    pub kind_of_filter: KindOfFilter,
    pub disabled: bool,
}

pub struct Filter {
    pub config: FilterConfig,
    buf0: i64,
    buf1: i64,
    feedback: u32,
}

impl Effect for Filter {
    fn clock(&mut self, sample: i16) -> i16 {
        if self.config.disabled {
            sample
        } else {
            self.filter(sample)
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
        let band = (self.buf0 - self.buf1) as i32;
        let high = sample as i64 - self.buf0;
        let x = fx_mul(self.feedback as i64, band);
        self.buf0 = self.buf0 + fx_mul(high + x, self.config.cutoff_frequency as i32);
        self.buf1 = self.buf1 + ifx_mul(band, self.config.cutoff_frequency) as i64;
        let out: i16;
        match self.config.kind_of_filter {
            KindOfFilter::Low => out = self.buf1 as i16,
            KindOfFilter::High => out = high as i16,
            KindOfFilter::Band => out = band as i16,
            KindOfFilter::Notch => out = (sample as i64 - self.buf0 + self.buf1) as i16,
        }
        return out;
    }
}
