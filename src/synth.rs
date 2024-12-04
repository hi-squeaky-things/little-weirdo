//use std::println;

use effect::Effect;

use self::{data::frequencies::MIDI2FREQ, filter::Filter, mixer::Mixer, patch::Patch};

pub mod data;
pub mod envelope;
pub mod filter;
pub mod math;
pub mod mixer;
pub mod oscillator;
pub mod patch;
pub mod patches;
pub mod effect;

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

///
/// Implementation of the LttL Weirdo Wavetable Synthesizer
/// 
impl Synth {
    /// Creates a new instance of the LttL Weirdo Wavetable Synthesizer.
    ///
    /// This method takes two parameters:
    ///
    /// - `sample_rate`: The sample rate of the synthesizer, in Hz.
    /// - `patch`: A `Patch` struct containing configuration data for the Synthesizer.
    ///
    /// It returns a new `Synth` instance with the specified configuration.
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
            voices: if patch.mono { 1 } else { 2 },
        }
    }

    ///
    ///  Loads a synth patch into the LttL Weirdo Wavetable Synthesizer engine, configuring all necessary components.
    ///
    /// This method takes one parameters:
    ///
    /// - `patch`: A `Patch` struct containing configuration data for the LttL Weirdo Wavetable Synthesizer engine.
    ///
    ///
    pub fn load_patch(&mut self, patch: Patch) {
        // Create oscillators for both voices with their respective frequencies and detunes
        let voice1_freq = 440; // The base frequency of the oscillator (A4)
        let voice2_freq = 440;
        let voice1_detune = patch.voice_1_detune; // Detune value for voice 1
        let voice2_detune = patch.voice_2_detune;

        // LFOs are used to modulate parameters, this one affects the pitch of voice 1
        let lfo1_waveform = oscillator::Waveform::SawTooth; // Type of wave used by the LFO
        let lfo1_freq = patch.lfo_1 as u16; // Frequency of the LFO (in Hz)
        self.voice1 = oscillator::Oscillator::new(
            voice1_freq,
            patch.voice_1,
            voice1_detune,
            self.sample_rate,
            patch.glide,
            patch.glide_rate,
        );
        self.voice2 = oscillator::Oscillator::new(
            voice2_freq,
            patch.voice_2,
            voice2_detune,
            self.sample_rate,
            patch.glide,
            patch.glide_rate,
        );

        // Voice envelopes control the volume of each voice over time
        let voice1_env_params = patch.voice_1_env; // Parameters for the voice 1 envelope (attack, decay, sustain, release)
        let voice2_env_params = patch.voice_2_env;

        self.lfo1 =
            oscillator::Oscillator::new(lfo1_freq, lfo1_waveform, 0, self.sample_rate, false, 1);

        self.voice1_envelope =
            envelope::EnvelopeGenerator::new(voice1_env_params, self.sample_rate);
        self.voice2_envelope =
            envelope::EnvelopeGenerator::new(voice2_env_params, self.sample_rate);

        // Filter configures how the sound is modified after being generated
        let filter_config = patch.filter_config;

        self.filter = Filter::new(self.sample_rate, filter_config);

        // The mixer controls the overall level of each voice and the main gain
        let mix_levels = (
            patch.voice_1_mix_level,
            patch.voice_2_mix_level,
            patch.lfo_1_mix_level,
        );
        let main_gain = patch.main_gain;

        self.mixer = Mixer::new(mix_levels.0, mix_levels.1, mix_levels.2, main_gain);
    }

    ///
    /// Returns a 16-bit sample value representing the synthesized audio signal.
    /// This function should be called every time an audio device requests a new sample, and it will compute the correct sample at the current time based on the internal state of the synthesizer and the desired sample rate.
    ///
    fn clock(&mut self) -> i16 {
        // Clock the envelopes for Voice 1 and Voice 2
        let envelope1 = self.voice1_envelope.clock();
        let envelope2 = self.voice2_envelope.clock();

        // Generate samples for each voice, taking into account gain settings
        let mut voice_1_sample =
            math::percentage(self.voice1.clock(), self.mixer.gain_voice_1 as i16);
        let voice_2_sample = math::percentage(self.voice2.clock(), self.mixer.gain_voice_2 as i16);

        // Clock the Low Frequency Oscillator (LFO) for Voice 1
        let lfo1 = self.lfo1.clock();

        // Apply LFO modulation to Voice 1 sample if enabled
        if self.mixer.gain_lfo_1 != 0 {
            voice_1_sample = ((voice_1_sample as i32 * lfo1 as i32) / (i16::MAX as i32 / 2)) as i16;
        }

        // Mix the two voices together, taking into account envelope and velocity settings
        let mut mix_and_max_gain = math::percentage(voice_1_sample, envelope1)
            + math::percentage(voice_2_sample, envelope2);

        // Apply velocity modulation to the mix
        mix_and_max_gain = math::percentage(mix_and_max_gain, self.mixer.velocity as i16);

        // Pass the mixed signal through the filter
        let filtered_signal = self.filter.clock(mix_and_max_gain);

        // Finally, apply main gain setting and return the final sample value
        mix_and_max_gain = math::percentage(filtered_signal, self.mixer.gain_main as i16);
        mix_and_max_gain
    }

    /// Let the LttL Weirdo Wavetable Synthesizer engine play a specific note on the right voice and with a velocity.
    ///
    /// # Parameters
    ///
    /// * `voice`: The voice number (0x00)
    /// * `note`: The MIDI note number (0-108)
    /// * `velocity`: The velocity of the note (0-127)
    pub fn note_on(&mut self, voice: u8, note: u8, velocity: u8) {
        // Cap note range between C0 and C8
        if note < 24 || note > 108 {
            return;
        }

        match voice {
            0x00 => {
                // Update the mixer velocity for this voice
                self.mixer.velocity = velocity;

                // If we have only one voice, play both voices with a detune
                if self.voices == 1 {
                    let mut freq: u16 = MIDI2FREQ[(note as i8 + self.voice1.detune) as usize];
                    // Update the frequency of the first voice
                    self.voice1.change_freq(freq);
                    // Open the gate for the first voice envelope
                    self.voice1_envelope.open_gate();

                    freq = MIDI2FREQ[(note as i8 + self.voice2.detune) as usize];
                    // Update the frequency of the second voice
                    self.voice2.change_freq(freq);
                    // Open the gate for the second voice envelope
                    self.voice2_envelope.open_gate();
                } else {
                    // If we have multiple voices, play them one by one
                    if self.voice_active_count > 1 {
                        // Reset the active voice count
                        self.voice_active_count = 0;
                    }
                    // Add a new note to the active voices array
                    self.voice_active[self.voice_active_count as usize] = note;

                    match self.voice_active_count {
                        0 => {
                            // Update the frequency of the first active voice (first voice)
                            let freq: u16 = MIDI2FREQ[(note as i8 + self.voice1.detune) as usize];
                            self.voice1.change_freq(freq);
                            // Open the gate for the first voice envelope
                            self.voice1_envelope.open_gate();
                        }
                        1 => {
                            // Update the frequency of the second active voice (second voice)
                            let freq = MIDI2FREQ[(note as i8 + self.voice2.detune) as usize];
                            self.voice2.change_freq(freq);
                            // Open the gate for the second voice envelope
                            self.voice2_envelope.open_gate();
                        }
                        _ => {}
                    };

                    // Increment the active voice count
                    self.voice_active_count += 1;
                }
            }
            _ => {} // Ignore other voices
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
    ///
    /// Returns a 16-bit sample value representing the synthesized audio signal.
    /// This function should be called every time an audio device requests a new sample, and it will compute the correct sample at the current time based on the internal state of the synthesizer and the desired sample rate.
    /// You need to compensate the delta time yourself.
    ///
    pub fn clock_and_output(&mut self) -> i16 {
        self.clock()
    }
}
