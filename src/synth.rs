//use std::println;



pub mod data;
pub mod envelope;
pub mod math;
pub mod mixer;
pub mod wavetable_oscillator;
pub mod patch;
pub mod patches;
pub mod effects;
use effects::{overdrive::Overdrive, Effect};

use self::{data::frequencies::MIDI2FREQ, effects::filter::Filter, mixer::Mixer, patch::Patch};


pub trait Clockable {
    fn clock(&mut self, sample: Option<i16>) -> i16 {
        match sample {
            Some(s) => s, // Use the provided sample
            None => 0,     // Default value if no sample is provided
        }
    }
}

pub struct Synth {
    voice1: wavetable_oscillator::WaveTableOscillator,
    voice2: wavetable_oscillator::WaveTableOscillator,
    voice3: wavetable_oscillator::WaveTableOscillator,
 
  //  lfo1: oscillator::WaveTableOscillator,
    voice1_envelope: envelope::EnvelopeGenerator,
    voice2_envelope: envelope::EnvelopeGenerator,
    voice3_envelope: envelope::EnvelopeGenerator,
    filter: Filter,
    overdrive: Overdrive,
    mixer: Mixer,
    velocity: u8,
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
            voice1: wavetable_oscillator::WaveTableOscillator::new(
                patch.voice_1,
                sample_rate
            ),
            voice2: wavetable_oscillator::WaveTableOscillator::new(
                patch.voice_2,
                sample_rate
            ),
            voice3: wavetable_oscillator::WaveTableOscillator::new(
                patch.voice_3,
                sample_rate
            ),
            voice1_envelope: envelope::EnvelopeGenerator::new(patch.voice_1_env, sample_rate),
            voice2_envelope: envelope::EnvelopeGenerator::new(patch.voice_2_env, sample_rate),
            voice3_envelope: envelope::EnvelopeGenerator::new(patch.voice_3_env, sample_rate),
          
            filter: Filter::new(sample_rate, patch.filter_config),
            mixer: Mixer::new(patch.mixer_config),
            overdrive: Overdrive::new(patch.overdrive_config),
            velocity: 0,
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
        self.voice1.reload(patch.voice_1);
        self.voice2.reload(patch.voice_2);
        self.voice3.reload(patch.voice_3);

       // self.lfo1.reload(oscillator::Waveform::SawTooth, 0, false, 1);
       // self.lfo1.change_freq(patch.lfo_1 as u16);

        self.voice1_envelope.reload(patch.voice_1_env);
        self.voice2_envelope.reload(patch.voice_2_env);
        self.voice3_envelope.reload(patch.voice_3_env);
        
        //effects
        self.filter.reload(patch.filter_config);
        self.overdrive.reload(patch.overdrive_config);

        //mix
        self.mixer.reload(patch.mixer_config);
    }

    ///
    /// Returns a 16-bit sample value representing the synthesized audio signal.
    /// This function should be called every time an audio device requests a new sample, and it will compute the correct sample at the current time based on the internal state of the synthesizer and the desired sample rate.
    ///
    fn clock(&mut self) -> i16 {
        // Clock the envelopes for Voice 1 and Voice 2
        let envelope1 = self.voice1_envelope.clock(None);
        let envelope2 = self.voice2_envelope.clock(None);
        let envelope3 = self.voice3_envelope.clock(None);

        // Generate samples for each voice, taking into account gain settings
        let voice_1_sample = math::percentage(self.voice1.clock(None), self.mixer.config.gain_voice_1 as i16);
        let voice_2_sample = math::percentage(self.voice2.clock(None), self.mixer.config.gain_voice_2 as i16);
        let voice_3_sample = math::percentage(self.voice3.clock(None), self.mixer.config.gain_voice_3 as i16);

       
        // Mix the three voices together, taking into account envelope and velocity settings
        let mut sound_mixing = math::percentage(voice_1_sample, envelope1)
            + math::percentage(voice_2_sample, envelope2) + math::percentage(voice_3_sample, envelope3);

        // Apply velocity to the mix
        sound_mixing = math::percentage(sound_mixing, self.velocity as i16);

        // Pass the mixed signal through the filter
        let filtered_signal = self.filter.clock(sound_mixing);

        // Finally, apply main gain setting and return the final sample value
        sound_mixing = math::percentage(filtered_signal, self.mixer.config.gain_main as i16);
        sound_mixing = self.overdrive.clock(sound_mixing); 
        sound_mixing
    }

    /// Let the LttL Weirdo Wavetable Synthesizer engine play a specific note on the right voice and with a velocity.
    ///
    /// # Parameters
    ///
    /// * `voice`: The voice number (0x00)
    /// * `note`: The MIDI note number (0-108)
    /// * `velocity`: The velocity of the note (0-127)
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        // Cap note range between C0 and C8
        if self.range_safeguard(note) { return };
        // Update the mixer velocity for this voice
        self.velocity = velocity;

        // If we have only one voice, play both voices with a detune
        let mut freq: u16 = MIDI2FREQ[(note as i8 + self.voice1.config.detune) as usize];
        // Update the frequency of the first voice
        self.voice1.change_freq(freq);
        // Open the gate for the first voice envelope
        self.voice1_envelope.open_gate();

        freq = MIDI2FREQ[(note as i8 + self.voice2.config.detune) as usize];
        // Update the frequency of the second voice
        self.voice2.change_freq(freq);
        // Open the gate for the second voice envelope
        self.voice2_envelope.open_gate();


        freq = MIDI2FREQ[(note as i8 + self.voice3.config.detune) as usize];
        // Update the frequency of the second voice
        self.voice3.change_freq(freq);
        // Open the gate for the second voice envelope
        self.voice3_envelope.open_gate();
    }

    pub fn note_off(&mut self, note: u8) {
        self.voice1_envelope.close_gate();
        self.voice2_envelope.close_gate();
        self.voice3_envelope.close_gate();
    }
    ///
    /// Returns a 16-bit sample value representing the synthesized audio signal.
    /// This function should be called every time an audio device requests a new sample, and it will compute the correct sample at the current time based on the internal state of the synthesizer and the desired sample rate.
    /// You need to compensate the delta time yourself.
    ///
    pub fn clock_and_output(&mut self) -> i16 {
        self.clock()
    }

    fn range_safeguard(&mut self, note: u8) -> bool {
        if note < 24 || note > 108 {
            return true
        }
        false
    }
}
