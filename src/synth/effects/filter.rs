//! Lowpass filter 
use crate::synth::effects::Effect;

#[derive(Copy, Clone)]
pub struct FilterConfig {
    pub cutoff_frequency: i16,
    pub filter_on: bool,
}

pub struct LowPassFilter {
    previous_sample: f32,
    alpha: f32,
    pub cutoff_frequency: i16,
    current_cutoff_frequency: i16,
    pub filter_on: bool,
    sample_rate: u16,
}

impl Effect for LowPassFilter {
    fn clock(&mut self, sample: i16) -> i16 {
        if self.filter_on {
            // put current sample through the low-pass filter
            self.lowpass_filter(sample)
        } else {
            sample
        }
    }
}

impl LowPassFilter {
    pub fn new(sample_rate: u16, config: FilterConfig) -> Self {
        let mut filter = Self {
            cutoff_frequency: config.cutoff_frequency,
            filter_on: config.filter_on,
            previous_sample: 0.0,
            alpha: 0.0,
            sample_rate,
            current_cutoff_frequency: config.cutoff_frequency,
        };
        filter.prepare_filter();
        filter
    }

    fn prepare_filter(&mut self) {
        let rc = 1.0 / (self.current_cutoff_frequency as f32 * 2.0 * core::f32::consts::PI);
        // time per sample
        let dt = 1.0 / self.sample_rate as f32;
        self.alpha = dt / (rc + dt);
    }


    pub fn change_freq(&mut self, cv: i16) {
        self.current_cutoff_frequency = self.cutoff_frequency + cv;
        self.prepare_filter();
    }

    fn lowpass_filter(&mut self, sample: i16) -> i16 {
        self.previous_sample =
            self.previous_sample + self.alpha * (sample as f32 - self.previous_sample);
        self.previous_sample as i16
    }
}
