extern crate alloc;
use alloc::{sync::Arc, vec::Vec};

use crate::synth::{data::frequencies::MIDI2FREQ, Clockable};

// Struct that holds multiple boxed wavetables
#[derive(Clone)]
pub struct BoxedSamples {
    data: Vec<BoxedSample>,
}

impl Default for BoxedSamples {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxedSamples {
    // Creates a new empty BoxedWavetables instance with capacity for 10 wavetables
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(10),
        }
    }

    // Adds a new wavetable to the collection
    pub fn add(&mut self, wt: BoxedSample) {
        self.data.push(wt);
    }
}

/// A boxed sample containing audio data as a vector of 16-bit signed integers.
#[derive(Clone)]
pub struct BoxedSample {
    /// The actual audio sample data.
    pub data: Vec<u8>,
}

impl BoxedSample {
    /// Creates a new `BoxedSample` from a vector of 8-bit unsigned integers.
    ///
    /// This constructor converts 16-bit little-endian samples from the input data
    /// into a vector of 16-bit signed integers for internal processing.
    pub fn new(data: Vec<u8>) -> Self {
        let init = Self { data };
        init
    }
}

/// A sampler that plays back audio samples at different speeds.
pub struct AudioSampler {
    /// Reference to the underlying audio sample data.
    sampler: Arc<BoxedSamples>,

    sample_id: u8,
    /// Current position in the audio sample data.
    counter: f32,
    /// Speed increment for advancing through the sample data.
    increment: f32,
    open: bool,
    length: u32,
    sample_rate: u16,
    is_drums: bool,

    loop_start: u32,
    loop_end: u32,
    loop_is_started: bool,
    one_shot: bool,
    base_key: u8,
}

impl Clockable for AudioSampler {
    /// Processes one clock cycle of the sampler.
    ///
    /// Advances the playback position based on the configured speed and increment,
    /// and returns the current sample value.
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        if !self.open {
            return 0;
        }
        self.counter += self.increment;
        if self.counter as u32 >= self.length {
            if !self.loop_is_started && !self.one_shot {
                self.loop_is_started = true;
                self.length = self.loop_end - self.loop_start;
            }
            if self.one_shot {
                self.open = false;
            }

            self.counter = 0.0;

            if self.loop_is_started && !self.one_shot {
                let pointer = self.loop_start as usize * 2;
                let b1 = (self.sampler.data[self.sample_id as usize].data[pointer + 1] as i16) << 8;
                let b2 = self.sampler.data[self.sample_id as usize].data[pointer] as i16;
                return b1 | b2;
            }
            return 0;
        }
        // Calculate initial pointer position (samples are 16-bit, so multiply by 2)
        let mut pointer = (self.counter * 2.0) as usize;

        if self.loop_is_started {
            // Add loop start offset
            pointer = (pointer + (self.loop_start as usize * 2)) as usize;

            // Check if we've reached the loop end
            let loop_end_pos = (self.loop_end * 2) as usize;
            if pointer >= loop_end_pos {
                pointer = loop_end_pos - 2; // Position before loop end
            }
        } else {
            // Check if we've reached the end of the sample
            let sample_len = self.sampler.data[self.sample_id as usize].data.len();
            if pointer >= sample_len - 1 {
                pointer = sample_len - 2; // Position before end
            }
        }

        // Ensure pointer is even (for 16-bit samples)
        if pointer % 2 != 0 && pointer > 0 {
            pointer -= 1;
        }

        let b1 = (self.sampler.data[self.sample_id as usize].data[pointer + 1] as i16) << 8;
        let b2 = self.sampler.data[self.sample_id as usize].data[pointer] as i16;
        b1 | b2
    }
}

impl AudioSampler {
    /// Creates a new sampler instance with the given sample rate and audio data.

    pub fn new(
        sample_rate: u16,
        sample_id: u8,
        sampler: Arc<BoxedSamples>,
        is_drums: bool,
        loop_start: u32,
        loop_end: u32,
        one_shot: bool,
        base_key: u8,
    ) -> Self {
        let mut audio_sampler = AudioSampler {
            sample_rate,
            sampler,
            sample_id,
            counter: 0.0,
            increment: 0.0,
            length: 0,
            open: false,
            is_drums,
            loop_is_started: false,
            loop_start,
            loop_end,
            one_shot,
            base_key,
        };
        audio_sampler.increment = 1.0; // sample_rate as f32 / (base_freq * 100.0);
        
         if audio_sampler.one_shot {
              audio_sampler.length = (audio_sampler.sampler.data[audio_sampler.sample_id as usize].data.len() / 2) as u32;
        } else {
              audio_sampler.length = audio_sampler.loop_start as u32;
        }
      


        if audio_sampler.loop_end == 0 {
            audio_sampler.loop_end = audio_sampler.length;
        }
        audio_sampler
    }

    pub fn set_note(&mut self, note: u8) {
        if self.is_drums  {
            self.sample_id = note - 36;
            self.length = self.sampler.data[self.sample_id as usize].data.len() as u32;
        } else {
            self.change_freq(MIDI2FREQ[note as usize]);
        }
    }

    pub fn open_gate(&mut self) {
        self.counter = 0.0;
        self.open = true;
        self.loop_is_started = false;
        if self.one_shot {
              self.length = (self.sampler.data[self.sample_id as usize].data.len() / 2) as u32;
        } else {
              self.length = self.loop_start as u32;
        }
    }

    pub fn close_gate(&mut self) {
        /*   self.counter = 0;
        self.open = false;*/
    }

    /// Changes the playback frequency by adjusting speed and increment values.
    ///
    /// This method maps specific frequencies to corresponding speed and increment settings
    /// to achieve desired pitch variations.
    pub fn change_freq(&mut self, freq: u16) {
        self.counter = 0.0;
        self.increment = freq as f32 / MIDI2FREQ[self.base_key as usize] as f32
    }

    pub fn reload(
        &mut self,
        is_drum: bool,
        sample_id: u8,
        loop_start: u32,
        loop_end: u32,
        one_shot: bool,
    ) {
        self.open = false;
        self.counter = 0.0;
        self.is_drums = is_drum;
        self.sample_id = sample_id;
        self.increment = 1.0; // sample_rate as f32 / (base_freq * 100.0);
        self.loop_is_started = false;
        self.loop_start = loop_start;
        self.loop_end = loop_end;
        self.one_shot = one_shot;

        if self.one_shot {
              self.length = (self.sampler.data[self.sample_id as usize].data.len() / 2) as u32;
        } else {
              self.length = self.loop_start as u32;
        }
    }
}
