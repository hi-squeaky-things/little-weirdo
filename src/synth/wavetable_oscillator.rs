//! WaveTableOscillator to generate sounds using Wavetable synthesis.
use super::data::wavetables;
use super::Clockable;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

const HEADROOM_DIVIDER: i16 = 2;

#[derive(PartialEq, Debug,Copy, Clone)]
pub enum Waveform {
    Sine,
    Square,
    SawTooth,
    Noise,
    Triangle,
    Square25,
    Square10,
    Bass,
    Piano,
    EightBit,
}

impl Waveform {
    fn get_waveform_lookup_table(waveform: &Waveform) -> &'static [i16; 600] {
        match waveform {
            Waveform::SawTooth => &wavetables::SAWTOOTH,
            Waveform::Square => &wavetables::SQUARE,
            Waveform::Triangle => &wavetables::TRIANGLE,
            Waveform::Sine => &wavetables::SINE,
            Waveform::Bass => &wavetables::BASS,
            Waveform::Piano => &wavetables::PIANO,
            Waveform::EightBit => &wavetables::EIGHT_BIT,
            Waveform::Square25 => &wavetables::SQUARE_25,
            Waveform::Square10 => &wavetables::SQUARE_10,
            Waveform::Noise => &wavetables::SINE, //will be generated in realtime, this is just a placeholder
        }
    }
}


#[derive(Copy, Clone)]
pub struct WaveTableOscillatorConfig {
    pub waveform: Waveform,
    pub glide: bool,
    pub glide_rate: u8,
    pub detune: i8,
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
    waveform_lookup_table: &'static [i16; 600],
    target_freq: u16,
    freq_step: i16,
}

impl Clockable for WaveTableOscillator {
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
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
        match &self.config.waveform {
            Waveform::Noise => {
                output = self
                    .random
                    .gen_range((i16::MIN / HEADROOM_DIVIDER)..(i16::MAX / HEADROOM_DIVIDER));
            }
            _ => {
                output = self.waveform_lookup_table[self.lookup_table[self.t as usize] as usize]
                    / HEADROOM_DIVIDER
            }
        }
        self.t = self.t + 1;
        output
    }
}

impl WaveTableOscillator {
    pub fn new(
       config: WaveTableOscillatorConfig,
       sample_rate: u16
    ) -> Self {
        let mut osc = Self {
            config,
            t: 0,
            one_shot_loop: sample_rate / config.freq + 1u16,
            freq_change: false,
            random: SmallRng::seed_from_u64(23702372039u64),
            sample_rate,
            lookup_table: [0u16; 3000],
            waveform_lookup_table: &wavetables::SINE,
            target_freq: config.freq,
            freq_step: 0,
        };
        osc.waveform_lookup_table = Waveform::get_waveform_lookup_table(&osc.config.waveform);
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
     self.waveform_lookup_table = Waveform::get_waveform_lookup_table(&self.config.waveform);
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
