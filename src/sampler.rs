extern crate alloc;
use alloc::sync::Arc;

use crate::{
    effects::{bitcrunch::Bitcrunch, delay::Delay, overdrive::Overdrive, Effect},
    math,
    sampler::audio_sampler::AudioSampler,
    synth::{
        envelope::{self, EnvelopConfiguration},
        Clockable,
    },
};

pub mod audio_sampler;
pub mod data;
pub mod patch;
use data::patches::{BoxedSamplerPatches, Patches};

/// Number of voices available in the samplers
pub const AMOUNT_OF_VOICES: usize = 8;
/// Number of output channels (stereo)
pub const AMOUNT_OF_OUTPUT_CHANNELS: usize = 2;

/// Main synthesizer struct that handles audio generation
pub struct Sampler {
    pub drums: bool,
    pub sample_map: u8,
    sampler_voices: [AudioSampler; AMOUNT_OF_VOICES],
    /// Array of envelope generators for shaping sound
    envelops: [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES],

    /// Array tracking active notes
    active_note: [u8; AMOUNT_OF_VOICES],
    /// Array tracking active velocities for round-robin
    active_velocity: [u8; AMOUNT_OF_VOICES],
    overdrive: Overdrive,
    bitcrunch: Bitcrunch,
    delay: Delay,
    pub patches: Arc<BoxedSamplerPatches>,
    current_patch: u8,
    
    /// Current round-robin counter for each voice
    round_robin_counter: [u8; AMOUNT_OF_VOICES],
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
    pub fn new(
        sample_rate: u16,
        patch_selected: u8,
        patches: alloc::sync::Arc<BoxedSamplerPatches>,
        samples: alloc::sync::Arc<audio_sampler::BoxedSamples>,
    ) -> Self {
        let patch = patches.get_patches_reference(patch_selected);
        Self {
            sample_map: patch.sample_map,
            drums: patch.drums,
            sampler_voices: Sampler::init_sampler_voices(
                sample_rate,
                Arc::clone(&samples),
                patch.drums,
                patch.sample_map,
                patch.loop_start,
                patch.loop_end,
                patch.one_shot,
                patch.base_key
            ),
            envelops: Sampler::init_envs(patch.env_config, sample_rate),
            active_note: [0; AMOUNT_OF_VOICES],
            active_velocity: [0; AMOUNT_OF_VOICES],
            overdrive: Overdrive::new(patch.overdrive_config),
            bitcrunch: Bitcrunch::new(patch.bitcrunch_config),
            delay: Delay::new(patch.delay_config),
            patches,
            current_patch: patch_selected,
            round_robin_counter: [0; AMOUNT_OF_VOICES],
        }
    }

    pub fn load_patch(&mut self, patch_selected: u8) {
        let patch = self.patches.get_patches_reference(patch_selected);
        self.current_patch = patch_selected;
        self.drums = patch.drums;
        self.sample_map = patch.sample_map;
        
        for i in 0..self.sampler_voices.len() {
            let sample_id = if patch.drums {
                i as u8
            } else {
                patch.sample_map
            };

            self.sampler_voices[i].reload(patch.drums, sample_id, patch.loop_start, patch.loop_end, patch.one_shot, patch.base_key);
            self.envelops[i].reload(patch.env_config);
        }

        self.overdrive.reload(patch.overdrive_config);
        self.delay.reload(patch.delay_config);
        self.bitcrunch.reload(patch.bitcrunch_config);
    }

    /// Initialize envelope generators with given parameters
    fn init_envs(config: EnvelopConfiguration, sample_rate: u16) -> [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES] {
        let envelops: [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES] =
            array_init::array_init(|_i: usize| {
                envelope::EnvelopeGenerator::new(config, sample_rate)
            });
        envelops
    }

    /// Initialize waveform oscillators with given parameters
    fn init_sampler_voices(
        sample_rate: u16,
        samples: Arc<audio_sampler::BoxedSamples>,
        drums: bool,
        sample_map: u8,
        loop_start: u32,
        loop_end: u32,
        one_shot: bool,
        base_key: u8,
    ) -> [AudioSampler; AMOUNT_OF_VOICES] {
        let voice_samplers: [AudioSampler; AMOUNT_OF_VOICES] =
            array_init::array_init(|i: usize| {
                AudioSampler::new(
                    sample_rate,
                    if drums { i as u8 } else { sample_map },
                    Arc::clone(&samples),
                    drums,
                    loop_start,
                    loop_end,
                    one_shot,
                    base_key
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
            generate_voices[i] = self.sampler_voices[i].clock(None);
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
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        // Cap note range between C0 and C8
        if self.range_safeguard(note) {
            return;
        };

        let id = self.add_note(note);
        if id != 255 {
            self.active_velocity[id] = velocity;

            let (_sample_id, base_key, loop_start, loop_end, one_shot, drums, final_sample_id, rr_enabled, rr_count) = {
                let patch = self.patches.get_patches_reference(self.current_patch);
                let (sample_id, base_key, loop_start, loop_end, one_shot) =
                    self.get_zone_params(note, patch);
                let final_sample_id = self.get_velocity_layer_sample(sample_id, velocity, patch);
                (
                    sample_id,
                    base_key,
                    loop_start,
                    loop_end,
                    one_shot,
                    patch.drums,
                    final_sample_id,
                    patch.round_robin && patch.round_robin_count != 0,
                    patch.round_robin_count,
                )
            };

            let rr_sample_id = if rr_enabled {
                let rr_index = self.round_robin_counter[id] as usize % rr_count as usize;
                self.round_robin_counter[id] = (self.round_robin_counter[id] + 1) % rr_count;
                final_sample_id + rr_index as u8
            } else {
                final_sample_id
            };

            self.sampler_voices[id].reload(
                drums,
                rr_sample_id,
                loop_start,
                loop_end,
                one_shot,
                base_key
            );

            self.sampler_voices[id].set_note(note);
            self.sampler_voices[id].open_gate();
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
                        zone.one_shot
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
            patch.one_shot
        )
    }

    /// Get velocity layer sample based on velocity
    pub fn get_velocity_layer_sample(&self, base_sample_id: u8, velocity: u8, patch: &patch::Patch) -> u8 {
        if !patch.velocity_layers || patch.num_velocity_layers == 0 {
            return base_sample_id;
        }
        
        // Calculate which velocity layer to use
        let layer_index = (velocity as usize) * patch.num_velocity_layers as usize / 128;
        let layer_index = layer_index.min(patch.num_velocity_layers as usize - 1);
        
        // Offset the sample ID by the layer index
        base_sample_id + layer_index as u8
    }

    /// Get round-robin sample based on current counter
    pub fn get_round_robin_sample(&mut self, base_sample_id: u8, voice_id: usize, patch: &patch::Patch) -> u8 {
        if !patch.round_robin || patch.round_robin_count == 0 {
            return base_sample_id;
        }

        let rr_index = self.round_robin_counter[voice_id] as usize % patch.round_robin_count as usize;
        self.round_robin_counter[voice_id] = (self.round_robin_counter[voice_id] + 1) % patch.round_robin_count;
        base_sample_id + rr_index as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effects::{
            bitcrunch::BitcrunchConfiguration,
            delay::DelayConfiguration,
            overdrive::{KindOfOverdrive, OverdriveConfiguration},
        },
        sampler::{audio_sampler::BoxedSamples, patch::Patch},
    };
    use alloc::{sync::Arc, vec::Vec};

    #[test]
    fn note_on_uses_current_patch_after_load_patch() {
        let patch_a = Patch {
            name: "A".into(),
            drums: false,
            sample_map: 0,
            loop_start: 0,
            loop_end: 0,
            one_shot: false,
            base_key: 60,
            overdrive_config: OverdriveConfiguration {
                threshold: 0,
                kind: KindOfOverdrive::Soft,
                enabled: false,
            },
            bitcrunch_config: BitcrunchConfiguration { enabled: false },
            delay_config: DelayConfiguration {
                enabled: false,
                delay_time: 0,
                mix_percentage: 0,
                feedback: false,
                feedback_percentage: 0,
            },
            env_config: EnvelopConfiguration {
                attack_time: 0,
                decay_time: 0,
                release_time: 0,
                sustain_level: 0,
            },
            zones: Vec::new(),
            velocity_layers: false,
            num_velocity_layers: 0,
            round_robin: false,
            round_robin_count: 0,
        };

        let patch_b = Patch {
            name: "B".into(),
            drums: false,
            sample_map: 7,
            loop_start: 0,
            loop_end: 0,
            one_shot: false,
            base_key: 60,
            overdrive_config: OverdriveConfiguration {
                threshold: 0,
                kind: KindOfOverdrive::Soft,
                enabled: false,
            },
            bitcrunch_config: BitcrunchConfiguration { enabled: false },
            delay_config: DelayConfiguration {
                enabled: false,
                delay_time: 0,
                mix_percentage: 0,
                feedback: false,
                feedback_percentage: 0,
            },
            env_config: EnvelopConfiguration {
                attack_time: 0,
                decay_time: 0,
                release_time: 0,
                sustain_level: 0,
            },
            zones: Vec::new(),
            velocity_layers: false,
            num_velocity_layers: 0,
            round_robin: false,
            round_robin_count: 0,
        };

        let mut patches = BoxedSamplerPatches::new();
        patches.add(data::patches::BoxedSamplerPatch::new(patch_a));
        patches.add(data::patches::BoxedSamplerPatch::new(patch_b));
        let patches = Arc::new(patches);

        let mut sampler = Sampler::new(44100, 0, Arc::clone(&patches), Arc::new(BoxedSamples::new()));
        sampler.load_patch(1);
        sampler.note_on(60, 100);

        assert_eq!(sampler.current_patch, 1);
        assert_eq!(sampler.sample_map, 7);
    }
}
