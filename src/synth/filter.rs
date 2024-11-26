#[derive(Copy, Clone)]
pub struct FilterConfig {
    pub cutoff_frequency: i16,
    pub filter_on: bool,
    pub _24db: bool,
}

pub struct Filter {
    previous_sample: f32,
    alpha: f32,
    pub cutoff_frequency: i16,
    current_cutoff_frequency: i16,
    pub filter_on: bool,
    sample_rate: u16,
    _24db: bool,
}

impl Filter {
    pub fn new(sample_rate: u16, config: FilterConfig) -> Self {
        let mut filter = Self {
            cutoff_frequency: config.cutoff_frequency,
            filter_on: config.filter_on,
            previous_sample: 0.0,
            alpha: 0.0,
            sample_rate,
            current_cutoff_frequency: config.cutoff_frequency,
            _24db: config._24db,
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
    pub fn clock(&mut self, mut sample: i16) -> i16 {
        if self.filter_on {
            // put current sample through the low-pass filter
            if self._24db {
                sample = self.lowpass_filter(sample);
            }
            self.lowpass_filter(sample)
        } else {
            sample
        }
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
