//use std::println;

use self::{data::frequencies::MIDI2FREQ, filter::Filter, mixer::Mixer, patch::Patch};

pub mod data;
pub mod envelope;
pub mod filter;
pub mod math;
pub mod mixer;
pub mod oscillator;
pub mod patch;

pub struct Synth {
    pub voice1: oscillator::Oscillator,
    voice2: oscillator::Oscillator,
    lfo1: oscillator::Oscillator,
    pub voice1_envelope: envelope::EnvelopeGenerator,
    voice2_envelope: envelope::EnvelopeGenerator,
    pub filter: Filter,
    pub mixer: Mixer,
    sample_rate: u16,
    voices: u8,
    voice_active_count: u8,
    voice_active: [u8; 2],
}

impl Synth {
    pub fn new(sample_rate: u16, patch: Patch) -> Synth {
        Synth {
            voice1: oscillator::Oscillator::new(
                440,
                patch.voice_1,
                patch.voice_1_detune,
                sample_rate,
                patch.glide,
                patch.glide_rate,
            ),
            voice2: oscillator::Oscillator::new(
                440,
                patch.voice_2,
                patch.voice_2_detune,
                sample_rate,
                patch.glide,
                patch.glide_rate,
            ),
            lfo1: oscillator::Oscillator::new(
                patch.lfo_1 as u16,
                oscillator::Waveform::Square,
                0,
                sample_rate,
                false,
                1,
            ),
            voice1_envelope: envelope::EnvelopeGenerator::new(patch.voice_1_env, sample_rate),
            voice2_envelope: envelope::EnvelopeGenerator::new(patch.voice_2_env, sample_rate),
            filter: Filter::new(sample_rate, patch.filter_config),
            mixer: Mixer::new(
                patch.voice_1_mix_level,
                patch.voice_2_mix_level,
                patch.lfo_1_mix_level,
                patch.main_gain,
            ),
            sample_rate,
            voice_active_count: 0,
            voice_active: [0, 0],
            voices: if patch.mono == true { 1 } else { 2 },
        }
    }

    pub fn load_patch(&mut self, patch: Patch) {
        self.voice1 = oscillator::Oscillator::new(
            440,
            patch.voice_1,
            patch.voice_1_detune,
            self.sample_rate,
            patch.glide,
            patch.glide_rate,
        );
        self.voice2 = oscillator::Oscillator::new(
            440,
            patch.voice_2,
            patch.voice_2_detune,
            self.sample_rate,
            patch.glide,
            patch.glide_rate,
        );
        self.lfo1 = oscillator::Oscillator::new(
            patch.lfo_1 as u16,
            oscillator::Waveform::SawTooth,
            0,
            self.sample_rate,
            false,
            1,
        );
        self.voice1_envelope =
            envelope::EnvelopeGenerator::new(patch.voice_1_env, self.sample_rate);
        self.voice2_envelope =
            envelope::EnvelopeGenerator::new(patch.voice_2_env, self.sample_rate);
        self.filter = Filter::new(self.sample_rate, patch.filter_config);
        self.mixer = Mixer::new(
            patch.voice_1_mix_level,
            patch.voice_2_mix_level,
            patch.lfo_1_mix_level,
            patch.main_gain,
        );
    }

    fn clock(&mut self) -> i16 {
        let envelope1 = self.voice1_envelope.clock();
        let envelope2 = self.voice2_envelope.clock();
        let mut voice_1_sample =
            math::percentage(self.voice1.clock(), self.mixer.gain_voice_1 as i16);
        let voice_2_sample =
            math::percentage(self.voice2.clock(), self.mixer.gain_voice_2 as i16);

        let lfo1 = self.lfo1.clock();
        if self.mixer.gain_lfo_1 != 0 {
            voice_1_sample = ((voice_1_sample as i32 * lfo1 as i32) / (i16::MAX as i32 / 2)) as i16;
        }
        let mut mix_and_max_gain = math::percentage(voice_1_sample, envelope1)
            + math::percentage(voice_2_sample, envelope2);
        mix_and_max_gain = math::percentage(mix_and_max_gain, self.mixer.velocity as i16);
        mix_and_max_gain = self.filter.clock(mix_and_max_gain);

        mix_and_max_gain = math::percentage(mix_and_max_gain, self.mixer.gain_main as i16);
        mix_and_max_gain
    }

    fn percentage(sample: i16, percentage: i16) -> i16 {
        if percentage > 0 {
            let sample_up_scale: i32 = sample as i32 * 1000;
            let new_sample: i16 = (((sample_up_scale / 100) * percentage as i32) / 1000) as i16;
            return new_sample;
        };
        0
    }

    pub fn note_on(&mut self, voice: u8, note: u8, velocity: u8) {
        // cap between C0 and C8
        if note < 24 || note > 108 {
            return;
        }
        match voice {
            0x00 => {
                self.mixer.velocity = velocity;
                if self.voices == 1 {
                    let mut freq: u16 = MIDI2FREQ[(note as i8 + self.voice1.detune) as usize];
                    self.voice1.change_freq(freq);
                    self.voice1_envelope.open_gate();
                    freq = MIDI2FREQ[(note as i8 + self.voice2.detune) as usize];
                    self.voice2.change_freq(freq);
                    self.voice2_envelope.open_gate();
                } else {
                    if self.voice_active_count > 1 {
                        self.voice_active_count = 0;
                    }
                    self.voice_active[self.voice_active_count as usize] = note;
                    match self.voice_active_count {
                        0 => {
                            let freq: u16 = MIDI2FREQ[(note as i8 + self.voice1.detune) as usize];
                            self.voice1.change_freq(freq);
                            self.voice1_envelope.open_gate();
                        }
                        1 => {
                            let freq = MIDI2FREQ[(note as i8 + self.voice2.detune) as usize];
                            self.voice2.change_freq(freq);
                            self.voice2_envelope.open_gate();
                        }
                        _ => {}
                    };
                    self.voice_active_count += 1;
                }
            }
            _ => {}
        }
    }

    pub fn note_off(&mut self, voice: u8, note: u8) {
        if note < 21 || note > 108 {
            return;
        }
        match voice {
            0x00 => {
                if self.voices == 1 {
                    self.voice1_envelope.close_gate();
                    self.voice2_envelope.close_gate();
                } else {
                    for i in 0..2 {
                        if self.voice_active[i as usize] == note {
                            match i {
                                0 => {
                                    self.voice1_envelope.close_gate();
                                    self.voice_active[i as usize] = 0;
                                }
                                1 => {
                                    self.voice2_envelope.close_gate();
                                    self.voice_active[i as usize] = 0;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        };
    }

    pub fn prepare(&self) {}

    pub fn clock_and_output(&mut self) -> i16 {
        self.clock()
    }
}
