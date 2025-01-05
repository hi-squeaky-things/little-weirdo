//use std::println;

pub mod data;
pub mod effects;
pub mod envelope;
pub mod math;
pub mod mixer;
pub mod patch;
pub mod router;
pub mod wavetable_oscillator;
use data::wavetables::SoundBank;
use effects::{overdrive::Overdrive, Effect};
use router::Router;
use wavetable_oscillator::WaveTableOscillator;

use self::{data::frequencies::MIDI2FREQ, effects::filter::Filter, mixer::Mixer, patch::Patch};

pub trait Clockable {
    fn clock(&mut self, sample: Option<i16>) -> i16 {
        match sample {
            Some(s) => s, // Use the provided sample
            None => 0,    // Default value if no sample is provided
        }
    }
}

pub const AMOUNT_OF_VOICES: usize = 8;
pub const AMOUNT_OF_OUTPUT_CHANNELS: usize = 2;

pub struct Synth {
    voices: [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES],
    envelops: [envelope::EnvelopeGenerator; AMOUNT_OF_VOICES],
    lfo: [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES / 2],
    router: Router,
    filter: Filter,
    overdrive: Overdrive,
    mixer: Mixer,
    velocity: u8,
    active_note: [u8; AMOUNT_OF_VOICES / 2],
    mono: bool,
    _soundbank: &'static SoundBank,
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
    pub fn new(sample_rate: u16, patch: &Patch, soundbank: &'static SoundBank) -> Self {
        Self {
            _soundbank: soundbank,
            voices: Synth::init_voices(sample_rate, soundbank, patch),
            envelops: Synth::init_envs(sample_rate, patch),
            lfo: Synth::init_lfos(sample_rate, soundbank, patch),
            filter: Filter::new(patch.filter_config),
            mixer: Mixer::new(patch.mixer_config),
            overdrive: Overdrive::new(patch.overdrive_config),
            router: Router::new(patch.routering_config),
            velocity: 0,
            active_note: [0; AMOUNT_OF_VOICES / 2],
            mono: patch.mono,
        }
    }

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

    fn init_voices(
        sample_rate: u16,
        soundbank: &'static SoundBank,
        patch: &Patch,
    ) -> [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES] {
        let voices: [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES] =
            array_init::array_init(|i: usize| {
                wavetable_oscillator::WaveTableOscillator::new(
                    patch.voices[i],
                    soundbank,
                    sample_rate,
                )
            });
        voices
    }

    fn init_lfos(
        sample_rate: u16,
        soundbank: &'static SoundBank,
        patch: &Patch,
    ) -> [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES / 2] {
        let voices: [wavetable_oscillator::WaveTableOscillator; AMOUNT_OF_VOICES / 2] =
            array_init::array_init(|i: usize| {
                wavetable_oscillator::WaveTableOscillator::new_lfo(
                    patch.lfos[i],
                    soundbank,
                    sample_rate,
                )
            });
        voices
    }

    ///
    ///  Loads a synth patch into the LttL Weirdo Wavetable Synthesizer engine, configuring all necessary components.
    ///
    /// This method takes one parameters:
    ///
    /// - `patch`: A `Patch` struct containing configuration data for the LttL Weirdo Wavetable Synthesizer engine.
    ///
    ///
    pub fn load_patch(&mut self, patch: &Patch) {
        self.mono = patch.mono;

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
    /// Returns a 16-bit sample value representing the synthesized audio signal.
    /// This function should be called every time an audio device requests a new sample, and it will compute the correct sample at the current time based on the internal state of the synthesizer and the desired sample rate.
    ///
    fn clock(&mut self) -> [i16; 2] {
        let mut generate_voices: [i16; AMOUNT_OF_VOICES] = [0; AMOUNT_OF_VOICES];
        let mut generate_lfos: [i16; AMOUNT_OF_VOICES / 2] = [0; AMOUNT_OF_VOICES / 2];
        let mut generate_env: [i16; AMOUNT_OF_VOICES] = [0; AMOUNT_OF_VOICES];
        let mut sound_mixing: [i16; AMOUNT_OF_OUTPUT_CHANNELS] = [0; AMOUNT_OF_OUTPUT_CHANNELS];
        // clock voices and envelops once
        for i in 0..AMOUNT_OF_VOICES {
            generate_voices[i] = self.voices[i].clock(None);
            generate_env[i] = self.envelops[i].clock(None);
        }
        for i in 0..AMOUNT_OF_VOICES / 2 {
            let lfo: i32 = self.lfo[i].clock(None) as i32;
            let lfo_percentage = ((lfo + i16::MAX as i32) as u32 * 100) / u16::MAX as u32;
            generate_lfos[i] = lfo_percentage as i16;
        }


        for i in 0..AMOUNT_OF_VOICES / 2 {
            if self.router.config.voice_to_lfo[i].enable {
                generate_voices[self.router.config.voice_to_lfo[i].voice as usize] = math::percentage(
                    generate_voices[self.router.config.voice_to_lfo[i].voice as usize],
                    generate_lfos[i],
                );
            }
      
        }

        // run and route voices through envelops and apply gain.
        for i in 0..AMOUNT_OF_VOICES {
            generate_voices[i] = math::percentage(
                generate_voices[i],
                generate_env[self.router.config.voices_to_envelop[i].env as usize],
            );
            generate_voices[i] = math::percentage(generate_voices[i], self.velocity as i16);
            generate_voices[i] =
                math::percentage(generate_voices[i], self.mixer.config.gain_voices[i] as i16);
            sound_mixing[0] = sound_mixing[0] + generate_voices[i];
        }

        sound_mixing[1] = sound_mixing[0];

        // Pass the mixed signal through the filter
        /*
        if self.router.config.lfo_to_filter {
            let lfo_filter = 1_000 + math::percentage(5_000, lfo_percentage as i16);
            if self.filter.config.cutoff_frequency != lfo_filter as u16 {
                let mut config = self.filter.config;
                config.cutoff_frequency = lfo_filter as u16;
                self.filter.reload(config);
            }
        }
        */
        sound_mixing[0] = self.filter.clock(sound_mixing[0]);

        // Finally, apply main gain setting and return the final sample value
        sound_mixing[0] = math::percentage(sound_mixing[0], self.mixer.config.gain_main as i16);
        sound_mixing[0] = self.overdrive.clock(sound_mixing[0]);
        [sound_mixing[0], sound_mixing[0]]
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
        if self.range_safeguard(note) {
            return;
        };
        // Update the mixer velocity for this voice
        self.velocity = velocity;

        if self.mono {
            for i in 0..AMOUNT_OF_VOICES {
                let freq: u16 = MIDI2FREQ[(note as i8 + self.voices[i].config.detune) as usize];
                // Update the frequency of the voices
                self.voices[i].change_freq(freq);
                // Open the gate for all voice envelops
                self.envelops[i].open_gate();
            }
        } else {
            let id = self.add_note(note);
            if id != 255 {
                // If we have only one voice, play both voices with a detune
                for i in 0..2 {
                    let freq: u16 =
                        MIDI2FREQ[(note as i8 + self.voices[id * 2 + i].config.detune) as usize];
                    // Update the frequency of the voices
                    self.voices[id * 2 + i].change_freq(freq);
                    // Open the gate for all voice envelops
                    self.envelops[id * 2 + i].open_gate();
                }
            }
        }
    }

    pub fn note_off(&mut self, note: u8) {
        if self.mono {
            for i in 0..AMOUNT_OF_VOICES {
                self.envelops[i].close_gate();
            }
        } else {
            let id = self.remove_note(note);
            if id != 255 {
                for i in 0..2 {
                    self.envelops[id * 2 + i].close_gate();
                }
            }
        }
    }

    fn add_note(&mut self, note: u8) -> usize {
        match self.active_note.iter().position(|n| n == &note) {
            Some(position) => {
                return position;
            }
            None => match self.active_note.iter().position(|n| n == &0) {
                Some(position) => {
                    self.active_note[position] = note;
                    return position;
                }
                None => {
                    return 255;
                }
            },
        }
    }

    fn remove_note(&mut self, note: u8) -> usize {
        match self.active_note.iter().position(|n| n == &note) {
            Some(position) => {
                self.active_note[position] = 0;
                return position;
            }
            None => {
                return 255;
            }
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

    pub fn change_main_volume(&mut self, velocity: u8) {
        self.mixer.config.gain_main = velocity;
    }

    pub fn change_cutoff(&mut self, velocity: u8) {
        let mut config = self.filter.config;
        config.cutoff_frequency = velocity as u16 * 255;
        self.filter.reload(config);
    }

    pub fn change_resonance(&mut self, velocity: u8) {
        let mut config = self.filter.config;
        config.resonance = velocity as u16 * 255;
        self.filter.reload(config);
    }

    fn range_safeguard(&mut self, note: u8) -> bool {
        if note < 24 || note > 108 {
            return true;
        }
        false
    }
}
