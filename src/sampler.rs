extern crate alloc;
use alloc::sync::Arc;

use crate::{
    effects::{Effect, bitcrunch::Bitcrunch, overdrive::{Overdrive, OverdriveConfiguration}}, math, sampler::{audio_sampler::AudioSampler, patch::Patch}, synth::Clockable
};

pub mod audio_sampler;
pub mod patch;

/// Number of voices available in the samplers
pub const AMOUNT_OF_VOICES: usize = 5;
/// Number of output channels (stereo)
pub const AMOUNT_OF_OUTPUT_CHANNELS: usize = 2;

/// Main synthesizer struct that handles audio generation
pub struct Sampler {
    sampler_voices: [AudioSampler; AMOUNT_OF_VOICES],
     /// Array tracking active notes
    active_note: [u8; AMOUNT_OF_VOICES],
    overdrive: Overdrive,
    bitcrunch: Bitcrunch,
}

///
/// Implementation of the LttL Weirdo Wavetable Synthesizer
///
impl Sampler {
    /// Creates a new instance of the LttL Weirdo Wavetable Synthesizer.
    ///
    /// # Arguments
    /// * `sample_rate` - The sample rate of the synthesizer, in Hz.
    /// * `patch` - A `Patch` struct containing configuration data for the Synthesizer.
    /// * `wavetables` - Shared reference to wavetables for oscillator waveforms
    ///
    /// # Returns
    /// A new `Synth` instance with the specified configuration.
    pub fn new(sample_rate: u16, patch: &Patch, samples: alloc::sync::Arc<audio_sampler::BoxedSamples>) -> Self {
      
        Self {
            sampler_voices: Sampler::init_sampler_voices(sample_rate, Arc::clone(&samples)),
            active_note: [0; AMOUNT_OF_VOICES],
            overdrive: Overdrive::new(patch.overdrive_config),
            bitcrunch: Bitcrunch::new(patch.bitcrunch_config),
        }
    }

    /// Initialize waveform oscillators with given parameters
    fn init_sampler_voices(
        sample_rate: u16,
        samples: Arc<audio_sampler::BoxedSamples>,
    ) -> [AudioSampler; AMOUNT_OF_VOICES] {
        let voice_samplers: [AudioSampler; AMOUNT_OF_VOICES] =
            array_init::array_init(|i: usize| {
                AudioSampler::new(sample_rate, i as u8, Arc::clone(&samples))
            });
        voice_samplers
    }

    ///
    /// Generates a stereo audio sample by processing all voices and applying effects.
    /// This function should be called every time an audio device requests a new sample.
    ///
    /// # Returns
    /// An array containing left and right channel samples
    fn clock(&mut self) -> [i16; 2] {
        // sampler
        let mut sound_mixing: [i16; AMOUNT_OF_OUTPUT_CHANNELS] = [0; AMOUNT_OF_OUTPUT_CHANNELS];

        for i in 0..AMOUNT_OF_VOICES {
            let mut sampler_sample = self.sampler_voices[i].clock(None);
            sampler_sample = math::percentage(sampler_sample, 100);
            sound_mixing[0] = sound_mixing[0] + math::percentage(sampler_sample, 10);
        }

        if self.bitcrunch.config.enabled {
            sound_mixing[0] = self.bitcrunch.clock(sound_mixing[0]);
        }

        if self.overdrive.config.enabled {
            sound_mixing[0] = self.overdrive.clock(sound_mixing[0]);
        }

        [sound_mixing[0], sound_mixing[0]]
    }

    /// Play a specific note on the synthesizer
    ///
    /// # Arguments
    /// * `note` - The MIDI note number (0-108)
    /// * `velocity` - The velocity of the note (0-127)
    pub fn note_on(&mut self, note: u8, _velocity: u8) {
        // Cap note range between C0 and C8
        if self.range_safeguard(note) {
            return;
        };

        let id = self.add_note(note);
        if id != 255 {
            self.sampler_voices[id].set_note(note);
            self.sampler_voices[id].open_gate();
        }
    }

    pub fn note_off(&mut self, note: u8) {
        if self.range_safeguard(note) {
            return;
        };
        let id = self.remove_note(note);
        if id != 255 {
            self.sampler_voices[id].close_gate();
        }
    }

 /// Add a note to the active notes list
    /// Returns the index of the note in the active notes array, or 255 if no space
    fn add_note(&mut self, note: u8) -> usize {
        match self.active_note.iter().position(|n| n == &note) {
            Some(position) => position,
            None => match self.active_note.iter().position(|n| n == &0) {
                Some(position) => {
                    if position < AMOUNT_OF_VOICES {
                        self.active_note[position] = note;
                       position
                    } else {
                        255
                    }
                }
                None => 255,
            },
        }
    }

    /// Remove a note from the active notes list
    /// Returns the index of the note that was removed, or 255 if not found
    fn remove_note(&mut self, note: u8) -> usize {
        match self.active_note.iter().position(|n| n == &note) {
            Some(position) => {
                self.active_note[position] = 0;
                 position
            }
            None => 255,
        }
    }

    ///
    /// Returns a 16-bit sample value representing the synthesized audio signal.
    /// This function should be called every time an audio device requests a new sample, and it will compute the correct sample at the current time based on the internal state of the synthesizer and the desired sample rate.
    /// You need to compensate the delta time yourself.
    ///
    pub fn clock_and_output(&mut self) -> [i16; 2] {
        self.clock()
    }

    /// Check if the note is within the valid range (C0 to C8)
    /// Returns true if the note is outside the valid range
    fn range_safeguard(&mut self, note: u8) -> bool {
        if !(35..=39).contains(&note) {
            return true;
        }
        false
    }
}
