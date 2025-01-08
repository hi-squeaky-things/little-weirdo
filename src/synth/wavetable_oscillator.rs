//! WaveTableOscillator to generate sounds using Wavetable synthesis.
use core::iter::Empty;

use super::data::wavetables::{BoxedWavetables, Wavetables};
use super::{Clockable};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
 
use serde::Deserialize;
 use serde::Serialize;

 extern crate alloc;
use alloc::{boxed::Box, rc::Rc};
 

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
    pub freq: u16,
}


pub struct WaveTableOscillator {
    pub config: WaveTableOscillatorConfig,
    t: u16,
    one_shot_loop: u16,
    freq_change: bool,
    random: SmallRng,
    sample_rate: u16,
    lookup_table: [u16; 3000],
    target_freq: u16,
    freq_step: i16,
    speed: u16,
    last_output: i16,
    speed_count: u16,
    wavetables: Rc<BoxedWavetables>,
}

impl Clockable for WaveTableOscillator {
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        self.speed_count = self.speed_count + 1;
        if self.speed == self.speed_count {
            self.speed_count = 0;
            if self.t == self.one_shot_loop {
                self.t = 0;
                if self.freq_change {
                    if self.config.freq != self.target_freq {
                        if self.config.freq.abs_diff(self.target_freq) < self.freq_step.abs() as u16 {
                            self.config.freq = self.target_freq;
                            self.freq_change = false;
                        } else {
                            self.config.freq = (self.config.freq as i16 + self.freq_step) as u16;
                        }
                    } else {
                        self.freq_change = false;
                    }
                    self.one_shot_loop = (self.sample_rate / self.config.freq as u16) as u16;
                    self.calculate_lookup_table();
                }
            };
            let output: i16;
            match &self.config.soundbank_index {
                255 => {
                    output = self
                        .random
                        .gen_range((i16::MIN + 1000)..(i16::MAX - 1000));
                }
                _ => {
                    let index = self.lookup_table[self.t as usize] as usize;
                    output = self.wavetables.get_wavetable_reference(0)[index];
                }
            }
            self.t = self.t + 1;
            self.last_output = output;
        }
        self.last_output
    }
}

impl WaveTableOscillator {

    pub fn new_lfo(config: WaveTableLoFreqOscillatorConfig, sample_rate: u16,  wavetables: Rc<BoxedWavetables>) -> Self {
        let new_config = WaveTableOscillatorConfig {
            soundbank_index: config.soundbank_index,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq: 400,
            freq_detune: 0,
        };
        let mut osc = Self::new(new_config, sample_rate, wavetables);
        osc.speed = 4 * config.time as u16;
        osc
    }

    pub fn new(
       config: WaveTableOscillatorConfig,
       sample_rate: u16,
       wavetables: Rc<BoxedWavetables>,
    ) -> Self {
        let mut osc: WaveTableOscillator = Self {
            config,
            t: 0,
            one_shot_loop: sample_rate / config.freq + 1u16,
            freq_change: false,
            random: SmallRng::seed_from_u64(23702372039u64),
            sample_rate,
            lookup_table: [0u16; 3000],
            target_freq: config.freq,
            freq_step: 0,
            last_output: 0,
            speed_count: 0,
            speed: 1,
            wavetables: wavetables,
        };
      //  if osc.config.soundbank_index != 255 {
         //  osc.waveform_lookup_table = osc.wavetables.get_wavetable_reference(osc.config.soundbank_index);
        //}
        osc.calculate_lookup_table();
        osc
    }

    fn calculate_lookup_table(&mut self) {
        let sample_rate_expanded: u32 = self.sample_rate as u32 * 10_000;
        let one_loop: u32 = sample_rate_expanded / self.config.freq as u32;
        let one_step_loop: u32 = one_loop / 600;
        let mut increase: u32 = 0;
        let steps: u16 = self.sample_rate / self.config.freq;
        for i in 0..steps {
            self.lookup_table[i as usize] = (increase / one_step_loop) as u16;
            increase += 10_000;
        }
    }

    /// Change the frequency of this oscillator. If glide is set, the oscillator will glide to the new freq. 
    ///
    /// # Parameters
    ///
    /// * `freq`: The new frequency of this oscillator
  
   pub fn reload(&mut self, config: WaveTableOscillatorConfig) {
     self.config = config;
     if self.config.soundbank_index as usize != 255 {
     //   self.waveform_lookup_table = self.wavetables.get_wavetable_reference(0);
     }
   }

    pub fn change_freq(&mut self, frequency: u16) {
        if self.config.freq != frequency {
            self.target_freq = frequency;
            if self.config.glide {
                self.freq_step = (frequency as i16 - self.config.freq as i16) / self.config.glide_rate as i16;
                if self.freq_step == 0 {
                    self.freq_step = (frequency as i16 - self.config.freq as i16) / 2;
                };
            } else {
                self.config.freq = frequency;
            }
            self.freq_change = true;
        }
    }
}
