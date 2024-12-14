//! Lowpass filter 
use crate::synth::effects::Effect;

#[derive(Copy, Clone)]
pub enum KindOfFilter{
    Low,
    High,
}


#[derive(Copy, Clone)]
pub struct FilterConfig {
    pub cutoff_frequency: i16,
    pub kind_of_filter: KindOfFilter,
    pub pass_through: bool,
}

pub struct Filter {
    config: FilterConfig,
    previous_sample: f32,
    alpha: f32,
    current_cutoff_frequency: i16,
    sample_rate: u16,
}

impl Effect for Filter {
    fn clock(&mut self, sample: i16) -> i16 {
        if self.config.pass_through {
            // put current sample through the low-pass filter
            self.lowpass_filter(sample)
        } else {
            sample
        }
    }
}

impl Filter {
    pub fn new(sample_rate: u16, config: FilterConfig) -> Self {
        let mut filter = Self {
            config: config,
            previous_sample: 0.0,
            alpha: 0.0,
            sample_rate,
            current_cutoff_frequency: config.cutoff_frequency,
        };
        filter.prepare_filter();
        filter
    }

    pub fn reload(&mut self, config: FilterConfig) {
        self.config = config;
        self.prepare_filter();
    }

    fn prepare_filter(&mut self) {
        let rc = 1.0 / (self.current_cutoff_frequency as f32 * 2.0 * core::f32::consts::PI);
        // time per sample
        let dt = 1.0 / self.sample_rate as f32;
        self.alpha = dt / (rc + dt);
    }


    pub fn change_freq(&mut self, cv: i16) {
        self.current_cutoff_frequency = self.config.cutoff_frequency + cv;
        self.prepare_filter();
    }

    fn lowpass_filter(&mut self, sample: i16) -> i16 {
        self.previous_sample =
            self.previous_sample + self.alpha * (sample as f32 - self.previous_sample);
        self.previous_sample as i16
    }
}
