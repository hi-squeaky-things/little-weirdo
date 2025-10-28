pub mod data;
pub mod effects;
pub mod envelope;
pub mod math;
pub mod mixer;
pub mod patch;
pub mod router;
pub mod sampler;
pub mod wavetable_oscillator;
use data::wavetables::BoxedWavetables;
use effects::bitcrunch::Bitcrunch;
use effects::{overdrive::Overdrive, Effect};
use patch::SynthMode;
use router::Router;
extern crate alloc;
use alloc::sync::Arc;

use self::{data::frequencies::MIDI2FREQ, effects::filter::Filter, mixer::Mixer, patch::Patch};

/// Trait for clockable components that can process samples
pub trait Clockable {
    /// Process a sample, returning either the provided sample or a default value
    fn clock(&mut self, sample: Option<i16>) -> i16 {
        match sample {
            Some(s) => s, // Use the provided sample
            None => 0,    // Default value if no sample is provided
        }
    }
}

/// Number of voices available in the synthesizer
pub const AMOUNT_OF_VOICES: usize = 8;
/// Number of output channels (stereo)
pub const AMOUNT_OF_OUTPUT_CHANNELS: usize = 2;

/// Main synthesizer struct that handles audio generation
pub struct Synth {
    /// Array of waveform oscillators for generating sounds
    voices: [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES],
    /// Array of envelope generators for shaping sound
    envelops: [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES],
    /// Array of Low-Frequency Oscillators for modulation
    lfo: [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES / 2],
    //  sampler: Sampler,
    /// Audio routing system
    router: Router,
    /// Filter effect for sound shaping
    filter: Filter,
    /// Overdrive effect for distortion
    overdrive: Overdrive,
    /// Bitcrunch effect for digital degradation
    bitcrunch: Bitcrunch,
    /// Mixer for combining audio signals
    mixer: Mixer,
    /// Velocity of the currently playing note
    velocity: u8,
    /// Array tracking active notes
    active_note: [u8; AMOUNT_OF_VOICES],
    /// Current operating mode of the synthesizer
    mode: SynthMode,
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
        patch: &Patch,
        wavetables: alloc::sync::Arc<BoxedWavetables>,
    ) -> Self {
        Self {
            voices: Synth::init_voices(sample_rate, patch, Arc::clone(&wavetables)),
            envelops: Synth::init_envs(sample_rate, patch),
            lfo: Synth::init_lfos(sample_rate, patch, Arc::clone(&wavetables)),
            //   sampler: Sampler::new(sample_rate, Rc::clone(&sample)),
            filter: Filter::new(patch.filter_config),
            mixer: Mixer::new(patch.mixer_config),
            overdrive: Overdrive::new(patch.overdrive_config),
            bitcrunch: Bitcrunch::new(patch.bitcrunch_config),
            router: Router::new(patch.routering_config),
            velocity: 0,
            active_note: [0; AMOUNT_OF_VOICES],
            mode: patch.synth_config.mode,
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
        wavetables: Arc<BoxedWavetables>,
    ) -> [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES] {
        let voices: [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES] =
            array_init::array_init(|i: usize| {
                wavetable_oscillator::WaveTableOscillator::new(
                    patch.voices[i],
                    sample_rate,
                    Arc::clone(&wavetables),
                )
            });
        voices
    }

    /// Initialize Low-Frequency Oscillators with given parameters
    fn init_lfos(
        sample_rate: u16,
        patch: &Patch,
        wavetables: Arc<BoxedWavetables>,
    ) -> [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES / 2] {
        let voices: [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES / 2] =
            array_init::array_init(|i: usize| {
                wavetable_oscillator::WaveTableOscillator::new_lfo(
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
    pub fn load_patch(&mut self, patch: &Patch) {
        self.mode = patch.synth_config.mode;

        for i in 0..AMOUNT_OF_VOICES {
            self.voices[i].reload(patch.voices[i]);
            self.envelops[i].reload(patch.envelops[i]);
        }

        //effects
        self.filter.reload(patch.filter_config);
        self.overdrive.reload(patch.overdrive_config);

        //mix
        self.mixer.reload(patch.mixer_config);
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

        // Route LFO modulation to voices
        for i in 0..AMOUNT_OF_VOICES / 2 {
            if self.router.config.lfo_to_voice[i].enable {
                for j in 0..2 {
                    if self.router.config.lfo_to_voice[i].voices[j] != 255 {
                        generate_voices[self.router.config.lfo_to_voice[i].voices[j] as usize] =
                            math::percentage(
                                generate_voices
                                    [self.router.config.lfo_to_voice[i].voices[j] as usize],
                                generate_lfos[i],
                            );
                    }
                }
            }
        }

        // Run and route voices through envelopes and apply gain
        for i in 0..AMOUNT_OF_VOICES {
            generate_voices[i] = math::percentage(
                generate_voices[i],
                generate_env[self.router.config.voices_to_envelop[i] as usize],
            );
            generate_voices[i] = math::percentage(generate_voices[i], self.velocity as i16);
            generate_voices[i] =
                math::percentage(generate_voices[i], self.mixer.config.gain_voices[i] as i16);
            sound_mixing[0] += generate_voices[i];
        }

        // sampler
        /*
        let mut sampler_sample = self.sampler.clock(None);
        sampler_sample = math::percentage(sampler_sample, generate_env[0]);
        sound_mixing[0] = sound_mixing[0] +  math::percentage(sampler_sample, 10);
        */

        // Stereo output (mono to stereo)
        sound_mixing[1] = sound_mixing[0];

        // Apply filter effect
        if self.router.config.lfo_to_freq {
            self.voices[0].manipulate_freq(
                generate_lfos[0] as u8,
                self.router.config.lfo_to_freq_amount,
            );
        }
        if self.router.config.lfo_to_filter {
            let lfo_filter = 1_000 + math::percentage(10_000, generate_lfos[0]);
            if self.filter.config.cutoff_frequency != lfo_filter as u16 {
                let mut config = self.filter.config;
                config.cutoff_frequency = lfo_filter as u16;
                self.filter.reload(config);
            }
        }

        // Apply filter to mixed signal
        sound_mixing[0] = self.filter.clock(sound_mixing[0]);

        // Apply final effects
        sound_mixing[0] = math::percentage(sound_mixing[0], self.mixer.config.gain_main as i16);
        sound_mixing[0] = self.overdrive.clock(sound_mixing[0]);
        sound_mixing[0] = self.bitcrunch.clock(sound_mixing[0]);
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
        // Update the mixer velocity for this voice
        self.velocity = velocity;

        let divider = self.mode as usize;

        let id = self.add_note(note);
        if id != 255 {
            // If we have only one voice, play both voices with a detune
            for i in 0..divider {
                let freq: u16 =
                    MIDI2FREQ[(note as i8 + self.voices[id * divider + i].config.detune) as usize];
                // Update the frequency of the voices
                self.voices[id * divider + i].change_freq(
                    (freq as i16 + self.voices[id * divider + i].config.freq_detune as i16) as u16,
                );
                //   self.sampler.change_freq(freq);
                // Open the gate for all voice envelops
                self.envelops[id * divider + i].open_gate();
            }
        }
    }

    pub fn note_off(&mut self, note: u8) {
        let divider = self.mode as usize;
        let id = self.remove_note(note);
        if id != 255 {
            for i in 0..divider {
                self.envelops[id * divider + i].close_gate();
            }
        }
    }

    /// Add a note to the active notes list
    /// Returns the index of the note in the active notes array, or 255 if no space
    fn add_note(&mut self, note: u8) -> usize {
        let amount_of_notes: usize = 8 / self.mode as usize;
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

    ///
    /// Returns a 16-bit sample value representing the synthesized audio signal.
    /// This function should be called every time an audio device requests a new sample, and it will compute the correct sample at the current time based on the internal state of the synthesizer and the desired sample rate.
    /// You need to compensate the delta time yourself.
    ///
    pub fn clock_and_output(&mut self) -> [i16; 2] {
        self.clock()
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
