//! WaveTableOscillator to generate sounds using Wavetable synthesis.

use super::data::wavetables::{BoxedWavetables, Wavetables};
use super::math::percentage;
use super::Clockable;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use serde::Deserialize;
use serde::Serialize;

extern crate alloc;
use alloc::sync::Arc;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct WaveTableLoFreqOscillatorConfig {
    pub soundbank_index: u8,
    pub time: u16, //   100 x seconde
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct WaveTableOscillatorConfig {
    pub soundbank_index: u8,
    pub glide: bool,
    pub glide_rate: u8,
    pub detune: i8,
    pub freq_detune: i8,
}

pub struct WaveTableOscillator {
    pub config: WaveTableOscillatorConfig,
    phase: u16,
    loop_end: u16,
    freq_changed: bool,
    random: SmallRng,
    sample_rate: u16,
    lookup_table: [u16; 3000],
    target_freq: u16,
    original_freq: u16,
    freq_step: i16,
    current_freq: u16,
    speed: u16,
    last_output: i16,
    speed_count: u16,
    wavetables: Arc<BoxedWavetables>,
}

impl Clockable for WaveTableOscillator {
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        self.speed_count += 1;
        if self.speed == self.speed_count {
            self.speed_count = 0;

            // Reset phase when reaching end
            if self.phase == self.loop_end {
                self.phase = 0;

                // Handle frequency changes
                if self.freq_changed {
                    if self.current_freq != self.target_freq {
                        let diff = self.target_freq as i16 - self.current_freq as i16;
                        let step_size = self.freq_step;

                        if diff.abs() < step_size.abs() {
                            self.current_freq = self.target_freq;
                            self.freq_changed = false;
                        } else {
                            self.current_freq = (self.current_freq as i16 + step_size) as u16;
                        }
                    } else {
                        self.freq_changed = false;
                    }

                    self.loop_end = self.sample_rate / self.current_freq;
                    self.calculate_lookup_table();
                }
            }

            // Generate output
            let output = match self.config.soundbank_index {
                255 => {
                    // Random noise
                    self.random
                        .random_range((i16::MIN + 1000)..(i16::MAX - 1000))
                }
                _ => {
                    // Wavetable lookup
                    let index = self.lookup_table[self.phase as usize] as usize;
                    self.wavetables
                        .get_wavetable_reference(self.config.soundbank_index)[index]
                }
            };

            self.phase += 1;
            self.last_output = output;
        }

        self.last_output
    }
}

impl WaveTableOscillator {
    pub fn new_lfo(
        config: WaveTableLoFreqOscillatorConfig,
        sample_rate: u16,
        wavetables: Arc<BoxedWavetables>,
    ) -> Self {
        let new_config = WaveTableOscillatorConfig {
            soundbank_index: config.soundbank_index,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq_detune: 0,
        };
        let mut osc = Self::new(new_config, sample_rate, wavetables);
        osc.speed = 4 * config.time;
        osc
    }

    pub fn new(
        config: WaveTableOscillatorConfig,
        sample_rate: u16,
        wavetables: Arc<BoxedWavetables>,
    ) -> Self {
        let mut osc = Self {
            config,
            phase: 0,
            loop_end: sample_rate / 440 + 1,
            freq_changed: false,
            random: SmallRng::seed_from_u64(23702372039u64),
            sample_rate,
            lookup_table: [0u16; 3000],
            target_freq: 440,
            original_freq: 440,
            current_freq: 440,
            freq_step: 0,
            last_output: 0,
            speed_count: 0,
            speed: 1,
            wavetables,
        };
        osc.calculate_lookup_table();
        osc
    }

    fn calculate_lookup_table(&mut self) {
        let sample_rate_expanded: u32 = self.sample_rate as u32 * 10_000;
        let one_loop: u32 = sample_rate_expanded / self.current_freq as u32;
        let one_step_loop: u32 = one_loop / 600;
        let mut increase: u32 = 0;
        let steps: u16 = self.sample_rate / self.current_freq;

        for i in 0..steps {
            self.lookup_table[i as usize] = (increase / one_step_loop) as u16;
            increase += 10_000;
        }
    }

    /// Reload configuration
    pub fn reload(&mut self, config: WaveTableOscillatorConfig) {
        self.config = config;
    }

    /// Manipulate frequency with percentage
    pub fn manipulate_freq(&mut self, perc: u8, amount: u16) {
        self.current_freq = self.original_freq + percentage(amount as i16, perc as i16) as u16;
        self.freq_changed = true;
    }

    /// Change frequency with optional glide
    pub fn change_freq(&mut self, frequency: u16) {
        if self.current_freq != frequency {
            self.target_freq = frequency;

            if self.config.glide {
                let diff = frequency as i16 - self.current_freq as i16;
                self.freq_step = diff / self.config.glide_rate as i16;
                // Ensure non-zero step
                if self.freq_step == 0 {
                    self.freq_step = diff / 2;
                }
            } else {
                self.current_freq = frequency;
            }

            self.original_freq = frequency;
            self.freq_changed = true;
        }
    }
}
