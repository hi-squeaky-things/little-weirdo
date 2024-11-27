use super::data::wavetables;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

const HEADROOM_DIVIDER: i16 = 2;

#[derive(PartialEq, Debug)]
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

pub struct Oscillator {
    waveform: Waveform,
    pub freq: u16,
    t: u16,
    one_shot_loop: u16,
    freq_change: bool,
    random: SmallRng,
    sample_rate: u16,
    pub detune: i8,
    lookup_table: [u16; 3000],
    waveform_lookup_table: &'static [i16; 600],
    target_freq: u16,
    freq_step: i16,
    glide: bool,
    glide_rate: u8,
}

impl Oscillator {
    pub fn new(
        freq: u16,
        waveform: Waveform,
        detune: i8,
        sample_rate: u16,
        glide: bool,
        glide_rate: u8,
    ) -> Self {
        let mut osc = Self {
            freq,
            waveform,
            t: 0,
            one_shot_loop: sample_rate / freq + 1u16,
            freq_change: false,
            random: SmallRng::seed_from_u64(23702372039u64),
            sample_rate,
            detune,
            lookup_table: [0u16; 3000],
            waveform_lookup_table: &wavetables::SINE,
            target_freq: freq,
            freq_step: 0,
            glide,
            glide_rate,
        };
        match osc.waveform {
            Waveform::SawTooth => osc.waveform_lookup_table = &wavetables::SAWTOOTH,
            Waveform::Square => osc.waveform_lookup_table = &wavetables::SQUARE,
            Waveform::Square10 => osc.waveform_lookup_table = &wavetables::SQUARE_10,
            Waveform::Square25 => osc.waveform_lookup_table = &wavetables::SQUARE_25,
            Waveform::Triangle => osc.waveform_lookup_table = &wavetables::TRIANGLE,
            Waveform::Sine => osc.waveform_lookup_table = &wavetables::SINE,
            Waveform::Bass => osc.waveform_lookup_table = &wavetables::BASS,
            Waveform::Piano => osc.waveform_lookup_table = &wavetables::PIANO,
            Waveform::EightBit => osc.waveform_lookup_table = &wavetables::EIGHT_BIT,
            _ => {}
        };
        osc.calculate_lookup_table();
        osc
    }

    fn calculate_lookup_table(&mut self) {
        let sample_rate_expanded: u32 = self.sample_rate as u32 * 10_000;
        let one_loop: u32 = sample_rate_expanded / self.freq as u32;
        let one_step_loop: u32 = one_loop / 600;
        let mut increase: u32 = 0;
        let steps: u16 = self.sample_rate / self.freq;
        for i in 0..steps {
            self.lookup_table[i as usize] = (increase / one_step_loop) as u16;
            increase += 10_000;
        }
    }

    pub fn change_freq(&mut self, freq: u16) {
        if self.freq != freq {
            self.target_freq = freq;
            if self.glide {
                self.freq_step = (freq as i16 - self.freq as i16) / self.glide_rate as i16;
                if self.freq_step == 0 {
                    self.freq_step = (freq as i16 - self.freq as i16) / 2;
                };
            } else {
                self.freq = freq;
            }
            self.freq_change = true;
        }
    }

    pub fn clock(&mut self) -> i16 {
        if self.t == self.one_shot_loop {
            self.t = 0;
            if self.freq_change {
                if self.freq != self.target_freq {
                    if self.freq.abs_diff(self.target_freq) < self.freq_step.abs() as u16 {
                        self.freq = self.target_freq;
                        self.freq_change = false;
                    } else {
                        self.freq = (self.freq as i16 + self.freq_step) as u16;
                    }
                } else {
                    self.freq_change = false;
                }
                self.one_shot_loop = (self.sample_rate / self.freq as u16) as u16;
                self.calculate_lookup_table();
            }
        };
        let output: i16;
        match &self.waveform {
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
