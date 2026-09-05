extern crate alloc;
use alloc::sync::Arc;

use crate::{
    effects::{bitcrunch::Bitcrunch, delay::Delay, overdrive::Overdrive, Effect},
    math,
    synth::{
        envelope::{self, EnvelopConfiguration},
        Clockable,
    },
    wavetable::sample_voice::SampleVoice,
};

pub mod sample_voice;
pub mod data;
pub mod patch;
use data::patches::{BoxedSamplerPatches, Patches};

/// Number of voices available in the samplers
pub const AMOUNT_OF_VOICES: usize = 8;
/// Number of output channels (stereo)
pub const AMOUNT_OF_OUTPUT_CHANNELS: usize = 2;

/// Main synthesizer struct that handles audio generation
pub struct WavetableSynth {
    sample_rate: u16,
    pub drums: bool,
    pub sample_map: u8,
    sample_voices: [SampleVoice; AMOUNT_OF_VOICES],
    /// Array of envelope generators for shaping sound
    envelops: [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES],

    /// Array tracking active notes
    active_note: [u8; AMOUNT_OF_VOICES],
    overdrive: Overdrive,
    bitcrunch: Bitcrunch,
    delay: Delay,
    pub patches: Arc<BoxedSamplerPatches>,
    pub current_patch: u8,
}

///
/// Implementation of the LttL Weirdo Wavetable Synthesizer
///
impl WavetableSynth {
    /// Creates a new instance of the LttL Weirdo Wavetable Synthesizer.
    ///
    /// # Arguments
    /// * `sample_rate` - The sample rate of the synthesizer, in Hz.
    /// * `patch` - A `Patch` struct containing configuration data for the Synthesizer.
    /// * `wavetables` - Shared reference to wavetables for oscillator waveforms
    ///
    /// # Returns
    /// A new `Synth` instance with the specified configuration.
    pub fn new(
        sample_rate: u16,
        patch_selected: u8,
        patches: alloc::sync::Arc<BoxedSamplerPatches>,
        samples: alloc::sync::Arc<sample_voice::BoxedSamples>,
    ) -> Self {
        let patch = patches.get_patches_reference(patch_selected);
        Self {
            sample_rate,
            sample_map: patch.sample_map,
            drums: patch.drums,
            sample_voices: WavetableSynth::init_sample_voices(
                sample_rate,
                Arc::clone(&samples),
                patch.drums,
                patch.sample_map,
                patch.loop_start,
                patch.loop_end,
                patch.one_shot,
                patch.base_key,
            ),
            envelops: WavetableSynth::init_envs(patch.env_config, sample_rate),
            active_note: [0; AMOUNT_OF_VOICES],
            overdrive: Overdrive::new(patch.overdrive_config),
            bitcrunch: Bitcrunch::new(patch.bitcrunch_config),
            delay: Delay::new(patch.delay_config, sample_rate),
            patches,
            current_patch: patch_selected,
        }
    }

    pub fn load_patch(&mut self, patch_selected: u8) {
        let patch = self.patches.get_patches_reference(patch_selected);
        self.current_patch = patch_selected;
        self.drums = patch.drums;
        self.sample_map = patch.sample_map;

        for i in 0..self.sample_voices.len() {
            let sample_id = if patch.drums {
                i as u8
            } else {
                patch.sample_map
            };

            self.sample_voices[i].reload(
                patch.drums,
                sample_id,
                patch.loop_start,
                patch.loop_end,
                patch.one_shot,
                patch.base_key,
            );
            self.envelops[i].reload(patch.env_config);
        }

        self.overdrive.reload(patch.overdrive_config);
        self.delay.reload(patch.delay_config, self.sample_rate);
        self.bitcrunch.reload(patch.bitcrunch_config);
    }

    /// Initialize envelope generators with given parameters
    fn init_envs(
        config: EnvelopConfiguration,
        sample_rate: u16,
    ) -> [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES] {
        let envelops: [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES] =
            array_init::array_init(|_i: usize| {
                envelope::EnvelopeGenerator::new(config, sample_rate)
            });
        envelops
    }

    /// Initialize waveform oscillators with given parameters
    fn init_sample_voices(
        sample_rate: u16,
        samples: Arc<sample_voice::BoxedSamples>,
        drums: bool,
        sample_map: u8,
        loop_start: u32,
        loop_end: u32,
        one_shot: bool,
        base_key: u8,
    ) -> [SampleVoice; AMOUNT_OF_VOICES] {
        let voice_samplers: [SampleVoice; AMOUNT_OF_VOICES] =
            array_init::array_init(|i: usize| {
                SampleVoice::new(
                    sample_rate,
                    if drums { i as u8 } else { sample_map },
                    Arc::clone(&samples),
                    drums,
                    loop_start,
                    loop_end,
                    one_shot,
                    base_key,
                )
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
        let mut generate_voices: [i16; AMOUNT_OF_VOICES] = [0; AMOUNT_OF_VOICES];
        let mut generate_env: [i16; AMOUNT_OF_VOICES] = [0; AMOUNT_OF_VOICES];

        for i in 0..AMOUNT_OF_VOICES {
            generate_voices[i] = self.sample_voices[i].clock(None);
            generate_env[i] = self.envelops[i].clock(None);
        }

        // Run and route voices through envelopes and apply gain
        for i in 0..AMOUNT_OF_VOICES {
            generate_voices[i] = math::percentage(generate_voices[i], generate_env[i]);
            generate_voices[i] = math::percentage(generate_voices[i], 25 as i16);
            sound_mixing[0] += generate_voices[i];
        }

        sound_mixing[0] = self.overdrive.clock(sound_mixing[0]);
        sound_mixing[0] = self.bitcrunch.clock(sound_mixing[0]);
        sound_mixing[0] = self.delay.clock(sound_mixing[0]);

        [sound_mixing[0], sound_mixing[0]]
    }

    /// Play a specific note on the synthesizer
    ///
    /// # Arguments
    /// * `note` - The MIDI note number (0-108)
    /// * `velocity` - The velocity of the note (0-127)
    ///
    /// TODO: Implement velocity
    ///

    pub fn note_on(&mut self, note: u8, _velocity: u8) {
        // Cap note range between C0 and C8
        if self.range_safeguard(note) {
            return;
        };

        let id = self.add_note(note);
        if id != 255 {
            self.delay.reset();
            let (sample_id, base_key, loop_start, loop_end, one_shot, drums) = {
                let patch = self.patches.get_patches_reference(self.current_patch);
                let (sample_id, base_key, loop_start, loop_end, one_shot) =
                    self.get_zone_params(note, patch);
                (
                    sample_id,
                    base_key,
                    loop_start,
                    loop_end,
                    one_shot,
                    patch.drums,
                )
            };

            self.sample_voices[id]
                .reload(drums, sample_id, loop_start, loop_end, one_shot, base_key);

            self.sample_voices[id].set_note(note);
            self.sample_voices[id].open_gate();
            self.envelops[id].open_gate();
        }
    }

    pub fn note_off(&mut self, note: u8) {
        if self.range_safeguard(note) {
            return;
        };
        let id = self.remove_note(note);
        if id != 255 {
            self.envelops[id].close_gate();
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
        if self.drums {
            if !(36..36 + 10).contains(&note) {
                return true;
            }
        } else {
            if !(24..=108).contains(&note) {
                return true;
            }
        }
        false
    }

    /// Get zone parameters based on the note
    /// Returns (sample_id, base_key, loop_start, loop_end, one_shot)
    pub fn get_zone_params(&self, note: u8, patch: &patch::Patch) -> (u8, u8, u32, u32, bool) {
        // Check if zones are defined
        if !patch.zones.is_empty() {
            // Find the first zone that contains this note
            for zone in &patch.zones {
                if note >= zone.start_note && note <= zone.end_note {
                    return (
                        zone.sample_map,
                        zone.base_key,
                        zone.loop_start,
                        zone.loop_end,
                        zone.one_shot,
                    );
                }
            }
        }

        // Fallback to default patch parameters
        (
            patch.sample_map,
            patch.base_key,
            patch.loop_start,
            patch.loop_end,
            patch.one_shot,
        )
    }
}
