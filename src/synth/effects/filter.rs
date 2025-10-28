//! Lowpass filter implementation
use serde::{Deserialize, Serialize};

use crate::synth::effects::Effect;
use crate::synth::math::{fx_mul, ifx_mul, ucfx_mul};

/// Types of filters available
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum KindOfFilter {
    Low,   // Low-pass filter
    High,  // High-pass filter
    Band,  // Band-pass filter
    Notch, // Notch filter
}

/// Configuration for the filter effect
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub cutoff_frequency: u16,        // Cutoff frequency (0-65535)
    pub resonance: u16,               // Resonance amount (0-65535)
    pub kind_of_filter: KindOfFilter, // Type of filter to apply
    pub enabled: bool,                // Whether the filter is active
}

/// Digital filter implementation
pub struct Filter {
    pub config: FilterConfig,
    buf0: i64,     // First buffer for filter state
    buf1: i64,     // Second buffer for filter state
    feedback: u32, // Feedback coefficient
}

impl Effect for Filter {
    /// Process a single audio sample through the filter
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
    /// Maximum u16 value

    /// Create a new filter with given configuration
    pub fn new(config: FilterConfig) -> Self {
        let mut filter = Self {
            config,
            buf0: 0,
            buf1: 0,
            feedback: 0,
        };
        filter.prepare_filter();
        filter
    }

    /// Update filter configuration and recompute parameters
    pub fn reload(&mut self, config: FilterConfig) {
        self.config = config;
        self.prepare_filter();
    }

    /// Prepare filter coefficients based on current configuration
    fn prepare_filter(&mut self) {
        // Calculate feedback coefficient using fixed-point arithmetic
        self.feedback = self.config.resonance as u32
            + ucfx_mul(
                self.config.resonance,
                Filter::SHIFTED_1 - self.config.cutoff_frequency,
            );
    }

    /// Apply the filter to an input sample
    fn filter(&mut self, sample: i16) -> i16 {
        // Convert sample to i64 for processing
        let sample_i64 = sample as i64;

        // Calculate highpass component (difference from previous buffer)
        let highpass = sample_i64 - self.buf0;

        // Calculate bandpass component (difference between buffers)
        let bandpass = (self.buf0 - self.buf1) as i32;

        // Apply feedback to bandpass signal
        let feedback_on_bandpass = fx_mul(self.feedback as i64, bandpass);

        // Update buffer states using fixed-point multiplication
        self.buf0 += fx_mul(
            highpass + feedback_on_bandpass,
            self.config.cutoff_frequency as i32,
        );
        self.buf1 += ifx_mul(bandpass, self.config.cutoff_frequency) as i64;

        // Select output based on filter type
        let out: i16 = match self.config.kind_of_filter {
            KindOfFilter::Low => self.buf1 as i16,
            KindOfFilter::High => highpass as i16,
            KindOfFilter::Band => bandpass as i16,
            KindOfFilter::Notch => (sample_i64 - self.buf0 + self.buf1) as i16,
        };

        out
    }
}
