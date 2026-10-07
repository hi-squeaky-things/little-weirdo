pub mod data;
pub mod envelope;
pub mod mixer;
pub mod noise;
pub mod patch;
pub mod router;
pub mod waveform_oscillator;
use data::waveforms::BoxedWaveforms;
use effects::Effect;
use patch::SynthMode;
use router::Router;
extern crate alloc;
use alloc::sync::Arc;

use crate::{
    effects::{self, bitcrunch::Bitcrunch, delay::Delay, flanger::Flanger, overdrive::Overdrive},
    effects::overdrive::{KindOfOverdrive, OverdriveConfiguration},
    math,
    synth::data::patches::{BoxedPatches, Patches},
};

use self::{data::frequencies::MIDI2FREQ, effects::filter::Filter, mixer::Mixer, patch::Patch};

/// Trait for clockable components that can process samples
pub trait Clockable {
    /// Process a sample, returning either the provided sample or a default value
    fn clock(&mut self, sample: Option<i16>) -> i16 {
        sample.unwrap_or_default()
    }
}

/// Number of voices available in the synthesizer
pub const AMOUNT_OF_VOICES: usize = 8;
/// Number of output channels (stereo)
pub const AMOUNT_OF_OUTPUT_CHANNELS: usize = 2;

/// Main synthesizer struct that handles audio generation
pub struct Synth {
    sample_rate: u16,
    /// Array of waveform oscillators for generating sounds
    voices: [waveform_oscillator::WaveformOscillator; AMOUNT_OF_VOICES],
    /// Array of envelope generators for shaping sound
    envelops: [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES],
    /// Array of Low-Frequency Oscillators for modulation
    lfo: [waveform_oscillator::WaveformOscillator; AMOUNT_OF_VOICES / 2],

    //  sampler: Sampler,
    /// Audio routing system
    router: Router,
    /// Filter effect for sound shaping
    filter: Filter,
    /// Overdrive effect for distortion
    overdrive: Overdrive,
    /// Bitcrunch effect for digital degradation
    bitcrunch: Bitcrunch,

    /// Delay effect
    delay: Delay,
    /// Flanger effect
    flanger: Flanger,
    /// Mixer for combining audio signals
    mixer: Mixer,
    /// Velocity of the currently playing note
    velocity: u8,
    /// Array tracking active notes
    active_note: [u8; AMOUNT_OF_VOICES],
    /// Current operating mode of the synthesizer
    mode: SynthMode,
    /// Reference to all the patches on the heap
    patches: Arc<BoxedPatches>,
}

///
/// Implementation of the LttL Weirdo Wavetable Synthesizer
///
impl Synth {
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
        patches: alloc::sync::Arc<BoxedPatches>,
        waveforms: alloc::sync::Arc<BoxedWaveforms>,
    ) -> Self {
        let patch = patches.get_patches_reference(patch_selected);
        Self {
            sample_rate,
            voices: Synth::init_voices(sample_rate, patch, Arc::clone(&waveforms)),
            envelops: Synth::init_envs(sample_rate, patch),
            lfo: Synth::init_lfos(sample_rate, patch, Arc::clone(&waveforms)),
            filter: Filter::new(patch.filter_config),
            mixer: Mixer::new(patch.mixer_config),
            overdrive: Overdrive::new(patch.overdrive_config),
            bitcrunch: Bitcrunch::new(patch.bitcrunch_config),
            delay: Delay::new(patch.delay_config, sample_rate),
            flanger: Flanger::new(patch.flanger_config, sample_rate),
            router: Router::new(patch.routering_config),
            velocity: 0,
            active_note: [0; AMOUNT_OF_VOICES],
            mode: patch.synth_config.mode,
            patches,
        }
    }

    /// Initialize envelope generators with given parameters
    fn init_envs(
        sample_rate: u16,
        patch: &Patch,
    ) -> [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES] {
        let envelops: [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES] =
            array_init::array_init(|i: usize| {
                envelope::EnvelopeGenerator::new(patch.envelops[i], sample_rate)
            });
        envelops
    }

    /// Initialize waveform oscillators with given parameters
    fn init_voices(
        sample_rate: u16,
        patch: &Patch,
        waveforms: Arc<BoxedWaveforms>,
    ) -> [waveform_oscillator::WaveformOscillator; AMOUNT_OF_VOICES] {
        let voices: [waveform_oscillator::WaveformOscillator; AMOUNT_OF_VOICES] =
            array_init::array_init(|i: usize| {
                waveform_oscillator::WaveformOscillator::new_with_seed(
                    patch.voices[i],
                    sample_rate,
                    Arc::clone(&waveforms),
                    23702372039u64.wrapping_add(i as u64),
                )
            });
        voices
    }

    /// Initialize Low-Frequency Oscillators with given parameters
    fn init_lfos(
        sample_rate: u16,
        patch: &Patch,
        wavetables: Arc<BoxedWaveforms>,
    ) -> [waveform_oscillator::WaveformOscillator; AMOUNT_OF_VOICES / 2] {
        let voices: [waveform_oscillator::WaveformOscillator; AMOUNT_OF_VOICES / 2] =
            array_init::array_init(|i: usize| {
                waveform_oscillator::WaveformOscillator::new_lfo(
                    patch.lfos[i],
                    sample_rate,
                    Arc::clone(&wavetables),
                )
            });
        voices
    }

    ///
    /// Loads a synth patch into the LttL Weirdo Wavetable Synthesizer engine, configuring all necessary components.
    ///
    /// # Arguments
    /// * `patch` - A `Patch` struct containing configuration data for the LttL Weirdo Wavetable Synthesizer engine.
    ///
    pub fn load_patch(&mut self, patch_selected: u8) {
        let patch = self.patches.get_patches_reference(patch_selected);
        self.mode = patch.synth_config.mode;

        for i in 0..self.voices.len() {
            self.voices[i].reload(patch.voices[i]);
            self.envelops[i].reload(patch.envelops[i]);
        }
        for i in 0..self.lfo.len() {
            self.lfo[i].reload_lfo(patch.lfos[i]);
        }

        self.router.reload(patch.routering_config);

        //effects
        self.filter.reload(patch.filter_config);
        self.overdrive.reload(patch.overdrive_config);
        self.delay.reload(patch.delay_config, self.sample_rate);
        self.flanger.reload(patch.flanger_config, self.sample_rate);
        self.bitcrunch.reload(patch.bitcrunch_config);

        //mix
        self.mixer.reload(patch.mixer_config);
    }

    pub fn all_note_off(&mut self) {
        self.active_note = [0; AMOUNT_OF_VOICES];
        for i in 0..AMOUNT_OF_VOICES {
            self.envelops[i].reset();
        }
    }

    ///
    /// Generates a stereo audio sample by processing all voices and applying effects.
    /// This function should be called every time an audio device requests a new sample.
    ///
    /// # Returns
    /// An array containing left and right channel samples
    fn clock(&mut self) -> [i16; 2] {
        // Pre-allocate arrays for generated signals

        let mut generate_voices: [i16; AMOUNT_OF_VOICES] = [0; AMOUNT_OF_VOICES];
        let mut generate_lfos: [i16; AMOUNT_OF_VOICES / 2] = [0; AMOUNT_OF_VOICES / 2];
        let mut generate_env: [i16; AMOUNT_OF_VOICES] = [0; AMOUNT_OF_VOICES];
        let mut sound_mixing: [i16; AMOUNT_OF_OUTPUT_CHANNELS] = [0; AMOUNT_OF_OUTPUT_CHANNELS];
        let mut voice_mix: i32 = 0;
        let mut mono_sound: i16;

        // Clock voices and envelopes once
        for i in 0..AMOUNT_OF_VOICES {
            generate_voices[i] = self.voices[i].clock(None);
            generate_env[i] = self.envelops[i].clock(None);
        }

        // Process LFOs
        for i in 0..AMOUNT_OF_VOICES / 2 {
            let lfo: i32 = self.lfo[i].clock(None) as i32;
            let lfo_percentage = ((lfo + i16::MAX as i32) as u32 * 100) / u16::MAX as u32;
            generate_lfos[i] = lfo_percentage as i16;
        }

        // Route LFO modulation (amplification) to voices
        // TODO: refactor, is unclear how to use it
        for i in 0..AMOUNT_OF_VOICES  {
            if self.router.config.lfo_to_voice[i] != -1 {
                let lfo_index =  self.router.config.lfo_to_voice[i].clamp(0, 3) as usize;
                    generate_voices[i] =
                            math::percentage(
                                generate_voices
                                    [i],
                                generate_lfos[lfo_index],
                            );
                    
                }
        }

        // Run and route voices through envelopes and apply gain
        let active_notes = self.active_note_count();
        self.mixer.update_avc(active_notes, self.sample_rate);
        for i in 0..AMOUNT_OF_VOICES {
            generate_voices[i] = math::percentage(
                generate_voices[i],
                generate_env[self.router.config.voices_to_envelop[i] as usize],
            );
            generate_voices[i] = math::percentage(generate_voices[i], self.velocity as i16);
            generate_voices[i] = self
                .mixer
                .apply_voice_gain(generate_voices[i], self.mixer.config.gain_voices[i]);
            voice_mix += generate_voices[i] as i32;
        }
        mono_sound = voice_mix.clamp(i16::MIN as i32, i16::MAX as i32) as i16;

        // Apply filter effect
        if self.router.config.lfo_to_freq != -1 && self.router.config.lfo_to_freq < 4 {
            self.voices[0].manipulate_freq(
                generate_lfos[self.router.config.lfo_to_freq as usize] as u8,
                self.router.config.lfo_to_freq_amount,
            );
        }
        if self.router.config.lfo_to_filter != -1 &&  self.router.config.lfo_to_filter < 4  {
            let lfo_filter = 1_000 + math::percentage(10_000, generate_lfos[self.router.config.lfo_to_filter as usize]);
            if self.filter.config.cutoff_frequency != lfo_filter as u16 {
                let mut config = self.filter.config;
                config.cutoff_frequency = lfo_filter as u16;
                self.filter.reload(config);
            }
        }

        // Apply final effects
        mono_sound = self.overdrive.clock(mono_sound);
        mono_sound = self.bitcrunch.clock(mono_sound);
        mono_sound = self.delay.clock(mono_sound);
        mono_sound = self.flanger.clock(mono_sound);

        // Apply filter to mixed signal
        mono_sound = self.filter.clock(mono_sound);
        mono_sound = math::percentage(mono_sound, self.mixer.config.gain_main as i16);

        // Apply panning
        let mut pan_percentage = self.mixer.config.balance; // pan_value is already in percentage (-100 to 100)
        let mut left_percentage = (100 - pan_percentage.max(0)).min(100);
        let mut right_percentage = (100 + pan_percentage.min(0)).max(0);

        if self.router.config.lfo_to_balance != -1  && self.router.config.lfo_to_balance < 4 {
            pan_percentage = (generate_lfos[self.router.config.lfo_to_balance as usize] as i16 * 2 - 100) as i8;
            left_percentage = (100 - pan_percentage.max(0)).min(100);
            right_percentage = (100 + pan_percentage.min(0)).max(0);
        }

        sound_mixing[0] = math::percentage(mono_sound, left_percentage as i16);
        sound_mixing[1] = math::percentage(mono_sound, right_percentage as i16);

        sound_mixing
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
        // Update the mixer velocity for this voice
        // TODO: Velocity is not used in a proper way, need fixing. Each voice need a velocity, not a general one.
        self.velocity = (velocity as u32 * 1_000 / 1_270) as u8;

        let voices_per_note = self.mode.voices_per_note();

        let id = self.add_note(note);
        if id != 255 {
            self.delay.reset();
            self.flanger.reset();
            // If we have only one voice, play both voices with a detune
            for i in 0..voices_per_note {
                let voice_index = id * voices_per_note + i;
                let env_index = self.router.config.voices_to_envelop[voice_index] as usize;
                let freq: u16 =
                    MIDI2FREQ[(note as i8 + self.voices[voice_index].config.detune) as usize];
                // Update the frequency of the voices
                self.voices[voice_index].change_freq(
                    (freq as i16 + self.voices[voice_index].config.freq_detune as i16) as u16,
                );
                // Open the gate for the envelope that this voice is routed to.
                self.envelops[env_index].open_gate();
            }
        }
    }

    pub fn note_off(&mut self, note: u8) {
        let voices_per_note = self.mode.voices_per_note();
        let id = self.remove_note(note);
        if id != 255 {
            for i in 0..voices_per_note {
                let voice_index = id * voices_per_note + i;
                let env_index = self.router.config.voices_to_envelop[voice_index] as usize;
                self.envelops[env_index].close_gate();
            }
        }
    }

    /// Add a note to the active notes list
    /// Returns the index of the note in the active notes array, or 255 if no space
    fn add_note(&mut self, note: u8) -> usize {
        let amount_of_notes: usize = self.mode.max_active_notes();
        match self.active_note.iter().position(|n| n == &note) {
            Some(position) => position,
            None => match self.active_note.iter().position(|n| n == &0) {
                Some(position) => {
                    if position < amount_of_notes {
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

    fn active_note_count(&self) -> usize {
        self.active_note.iter().filter(|note| **note != 0).count()
    }

    ///
    /// Returns a 16-bit sample value representing the synthesized audio signal.
    /// This function should be called every time an audio device requests a new sample, and it will compute the correct sample at the current time based on the internal state of the synthesizer and the desired sample rate.
    /// You need to compensate the delta time yourself.
    ///
    pub fn clock_and_output(&mut self) -> [i16; 2] {
        self.clock()
    }


    pub fn change_all_voices( &mut self, voice_id: u8) 
     {
         for i in 0..AMOUNT_OF_VOICES {
             let mut config = self.voices[i].config;
             config.soundbank_index = voice_id;
             self.voices[i].reload(config);
         }
    }

    /// Change the settings of all envelopes
    ///
    /// # Arguments
    /// * `attack` - Optional attack time (0-255), defaults to current value
    /// * `decay` - Optional decay time (0-255), defaults to current value
    /// * `sustain` - Optional sustain level (0-255), defaults to current value
    /// * `release` - Optional release time (0-255), defaults to current value
    pub fn change_all_envelopes(
        &mut self,
        attack: Option<i16>,
        decay: Option<i16>,
        sustain: Option<i16>,
        release: Option<i16>,
    ) {
        for i in 0..AMOUNT_OF_VOICES {
            let mut config = self.envelops[i].configuration;

            if let Some(val) = attack {
                config.attack_time = val;
            }
            if let Some(val) = decay {
                config.decay_time = val;
            }
            if let Some(val) = sustain {
                config.sustain_level = val;
            }
            if let Some(val) = release {
                config.release_time = val;
            }

            self.envelops[i].update_configuration(config);
        }
    }

    pub fn overdrive_settings(&self) -> (bool, u8, u16) {
        let kind = match self.overdrive.config.kind {
            KindOfOverdrive::Hard => 0,
            KindOfOverdrive::Soft => 1,
            KindOfOverdrive::Softer => 2,
        };
        (
            self.overdrive.config.enabled,
            kind,
            self.overdrive.config.threshold,
        )
    }

    pub fn toggle_overdrive(&mut self) {
        let mut config = self.overdrive.config;
        config.enabled = !config.enabled;
        self.overdrive.reload(config);
    }

    pub fn cycle_overdrive_kind(&mut self, clockwise: bool) {
        let mut config = self.overdrive.config;
        config.kind = match (config.kind, clockwise) {
            (KindOfOverdrive::Hard, true) | (KindOfOverdrive::Softer, false) => KindOfOverdrive::Soft,
            (KindOfOverdrive::Soft, true) => KindOfOverdrive::Softer,
            (KindOfOverdrive::Soft, false) => KindOfOverdrive::Hard,
            (KindOfOverdrive::Softer, true) => KindOfOverdrive::Hard,
            (KindOfOverdrive::Hard, false) => KindOfOverdrive::Softer,
        };
        self.overdrive.reload(config);
    }

    pub fn adjust_overdrive_threshold(&mut self, clockwise: bool) {
        const THRESHOLD_STEP: u16 = 250;
        let mut config: OverdriveConfiguration = self.overdrive.config;
        config.threshold = if clockwise {
            config.threshold.saturating_add(THRESHOLD_STEP).min(i16::MAX as u16)
        } else {
            config.threshold.saturating_sub(THRESHOLD_STEP)
        };
        self.overdrive.reload(config);
    }

    pub fn delay_settings(&self) -> (bool, u16, u8, u8) {
        (
            self.delay.config.enabled,
            self.delay.config.delay_time,
            self.delay.config.mix_percentage,
            if self.delay.config.feedback {
                self.delay.config.feedback_percentage
            } else {
                0
            },
        )
    }

    pub fn toggle_delay(&mut self) {
        let mut config = self.delay.config;
        config.enabled = !config.enabled;
        self.delay.reload(config, self.sample_rate);
    }

    pub fn adjust_delay_time(&mut self, clockwise: bool) {
        const DELAY_TIME_STEP_MS: u16 = 10;
        let mut config = self.delay.config;
        config.delay_time = if clockwise {
            config.delay_time.saturating_add(DELAY_TIME_STEP_MS).min(5000)
        } else {
            config.delay_time.saturating_sub(DELAY_TIME_STEP_MS).max(DELAY_TIME_STEP_MS)
        };
        self.delay.reload(config, self.sample_rate);
    }

    pub fn adjust_delay_mix(&mut self, clockwise: bool) {
        let mut config = self.delay.config;
        config.mix_percentage = if clockwise {
            config.mix_percentage.saturating_add(1).min(100)
        } else {
            config.mix_percentage.saturating_sub(1)
        };
        self.delay.reload(config, self.sample_rate);
    }

    pub fn adjust_delay_feedback(&mut self, clockwise: bool) {
        let mut config = self.delay.config;
        let current_feedback = if config.feedback {
            config.feedback_percentage
        } else {
            0
        };
        config.feedback_percentage = if clockwise {
            current_feedback.saturating_add(1).min(100)
        } else {
            current_feedback.saturating_sub(1)
        };
        config.feedback = config.feedback_percentage != 0;
        self.delay.reload(config, self.sample_rate);
    }

    /// Change the main volume of the synthesizer
    pub fn change_main_volume(&mut self, velocity: u8) {
        self.mixer.config.gain_main = velocity;
    }

    /// Change the filter cutoff frequency
    pub fn change_cutoff(&mut self, velocity: u8) {
        let mut config = self.filter.config;
        config.cutoff_frequency = velocity as u16 * 255;
        self.filter.reload(config);
    }

    /// Change the filter resonance
    pub fn change_resonance(&mut self, velocity: u8) {
        let mut config = self.filter.config;
        config.resonance = velocity as u16 * 255;
        self.filter.reload(config);
    }

    /// Check if the note is within the valid range (C0 to C8)
    /// Returns true if the note is outside the valid range
    fn range_safeguard(&mut self, note: u8) -> bool {
        if !(24..=108).contains(&note) {
            return true;
        }
        false
    }
}
