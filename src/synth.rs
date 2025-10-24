//use std::println;

pub mod data;
pub mod effects;
pub mod envelope;
pub mod math;
pub mod mixer;
pub mod patch;
pub mod router;
pub mod wavetable_oscillator;
pub mod sampler;
use data::wavetables::BoxedWavetables;
use effects::bitcrunch::Bitcrunch;
use effects::{overdrive::Overdrive, Effect};
use patch::SynthMode;
use router::Router;
extern crate alloc;
use alloc::sync::Arc;

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
  //  sampler: Sampler,
    router: Router,
    filter: Filter,
    overdrive: Overdrive,
    bitcrunch: Bitcrunch,
    mixer: Mixer,
    velocity: u8,
    active_note: [u8; AMOUNT_OF_VOICES],
    mode: SynthMode,
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
    pub fn new(sample_rate: u16, patch: &Patch, wavetables: alloc::sync::Arc<BoxedWavetables>) -> Self {
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
        patch: &Patch,
        wavetables: Arc<BoxedWavetables>
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

    fn init_lfos(
        sample_rate: u16,
        patch: &Patch,
        wavetables: Arc<BoxedWavetables>
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
    ///  Loads a synth patch into the LttL Weirdo Wavetable Synthesizer engine, configuring all necessary components.
    ///
    /// This method takes one parameters:
    ///
    /// - `patch`: A `Patch` struct containing configuration data for the LttL Weirdo Wavetable Synthesizer engine.
    ///
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
                for j in 0..2 {
                    if self.router.config.voice_to_lfo[i].voices[j] != 255 {
                        generate_voices[self.router.config.voice_to_lfo[i].voices[j] as usize] =
                            math::percentage(
                                generate_voices
                                    [self.router.config.voice_to_lfo[i].voices[j] as usize],
                                generate_lfos[i],
                            );
                    }
                }
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

        // sampler
        /* 
        let mut sampler_sample = self.sampler.clock(None);
        sampler_sample = math::percentage(sampler_sample, generate_env[0]);
        sound_mixing[0] = sound_mixing[0] +  math::percentage(sampler_sample, 10);
        */

        
        sound_mixing[1] = sound_mixing[0];

        // Pass the mixed signal through the filter

        if self.router.config.lfo_to_freq {
            self.voices[0].manipulate_freq( generate_lfos[0] as u8, self.router.config.lfo_to_freq_amount);
       
        }
        if self.router.config.lfo_to_filter {
            let lfo_filter = 1_000 + math::percentage(10_000, generate_lfos[0] as i16);
            if self.filter.config.cutoff_frequency != lfo_filter as u16 {
                let mut config = self.filter.config;
                config.cutoff_frequency = lfo_filter as u16;
                self.filter.reload(config);
            }
        }

        sound_mixing[0] = self.filter.clock(sound_mixing[0]);

        // Finally, apply main gain setting and return the final sample value
        sound_mixing[0] = math::percentage(sound_mixing[0], self.mixer.config.gain_main as i16);
        sound_mixing[0] = self.overdrive.clock(sound_mixing[0]);
        sound_mixing[0] = self.bitcrunch.clock(sound_mixing[0]);
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

    fn add_note(&mut self, note: u8) -> usize {
        let amount_of_notes:usize = 8 / self.mode as usize;
        match self.active_note.iter().position(|n| n == &note) {
            Some(position) => {
                return position;
            }
            None => match self.active_note.iter().position(|n| n == &0) {
                Some(position) => {
                    if position < amount_of_notes {
                        self.active_note[position] = note;
                        return position;
                    } else {
                        return 255;
                    }
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
