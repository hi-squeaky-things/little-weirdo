extern crate alloc;
use alloc::{sync::Arc, vec::Vec};

use crate::synth::{data::frequencies::MIDI2FREQ, Clockable};

// Holds multiple boxed audio samples.
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
    // Creates an empty sample collection with capacity for 10 samples.
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(10),
        }
    }

    // Adds a sample to the collection.
    pub fn add(&mut self, sample: BoxedSample) {
        self.data.push(sample);
    }
}

/// A boxed sample containing raw audio data as little-endian bytes.
#[derive(Clone)]
pub struct BoxedSample {
    /// The actual audio sample data.
    pub data: Vec<u8>,
}

impl BoxedSample {
    /// Creates a new `BoxedSample` from raw audio data.
    pub fn new(data: Vec<u8>) -> Self {
        let init = Self { data };
        init
    }
}

/// A voice that plays back an audio sample at variable speeds.
#[allow(dead_code)]
pub struct SampleVoice {
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

impl Clockable for SampleVoice {
    /// Processes one clock cycle of the sample voice.
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

impl SampleVoice {
    /// Creates a new, closed `SampleVoice` with the given sample configuration.
    ///
    /// For one-shot playback, the voice length is derived from the selected
    /// sample's byte length. For looping playback, `loop_start` initially
    /// defines the end of the non-looping portion. A `loop_end` value of zero
    /// uses that initial length as the loop end.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The audio rate associated with the sample data.
    /// * `sample_id` - The index of the sample in `sampler`.
    /// * `sampler` - The shared collection containing the sample data.
    /// * `is_drums` - Whether MIDI notes select samples instead of changing pitch.
    /// * `loop_start` - The sample position where looping begins.
    /// * `loop_end` - The exclusive sample position where looping ends, or zero
    ///   to use the derived initial length.
    /// * `one_shot` - Whether playback stops after one pass instead of looping.
    /// * `base_key` - The MIDI note used as the sample's original pitch.
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
        let mut audio_sampler = SampleVoice {
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
            audio_sampler.length = (audio_sampler.sampler.data[audio_sampler.sample_id as usize]
                .data
                .len()
                / 2) as u32;
        } else {
            audio_sampler.length = audio_sampler.loop_start as u32;
        }

        if audio_sampler.loop_end == 0 {
            audio_sampler.loop_end = audio_sampler.length;
        }
        audio_sampler
    }

    /// Selects a note for playback.
    ///
    /// In drum mode, the note is converted to a sample index by subtracting
    /// MIDI note 36. Otherwise, the note's frequency is used to update the
    /// playback increment relative to `base_key` and the playback position is
    /// reset.
    pub fn set_note(&mut self, note: u8) {
        if self.is_drums {
            self.sample_id = note - 36;
            self.length = self.sampler.data[self.sample_id as usize].data.len() as u32;
        } else {
            self.change_freq(MIDI2FREQ[note as usize]);
        }
    }

    /// Opens the voice gate and restarts playback from the beginning.
    ///
    /// This resets the sample position and loop state. The active length is
    /// recalculated so a one-shot uses the full sample and a looping voice
    /// plays its pre-loop section before entering the loop.
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

    /// Closes the voice gate.
    ///
    /// Gate closing is currently reserved for future release behavior and
    /// does not stop or otherwise modify playback.
    pub fn close_gate(&mut self) {
        /*   self.counter = 0;
        self.open = false;*/
    }

    /// Changes the playback frequency by adjusting speed and increment values.
    ///
    /// The increment is calculated as `freq / base_frequency`, where the base
    /// frequency is the frequency of `base_key`. The playback position is reset
    /// before the new frequency takes effect.
    ///
    /// # Arguments
    ///
    /// * `freq` - The target frequency in hertz.
    pub fn change_freq(&mut self, freq: u16) {
        self.counter = 0.0;
        self.increment = freq as f32 / MIDI2FREQ[self.base_key as usize] as f32
    }

    /// Stops playback and replaces the voice's sample configuration.
    ///
    /// The voice remains closed until [`SampleVoice::open_gate`] is called.
    /// Playback speed, loop state, selected sample, drum mode, and pitch
    /// settings are reset from the supplied arguments. The sample collection
    /// itself is shared and is not modified.
    ///
    /// # Arguments
    ///
    /// * `is_drum` - Whether MIDI notes select samples instead of changing pitch.
    /// * `sample_id` - The index of the replacement sample.
    /// * `loop_start` - The sample position where looping begins.
    /// * `loop_end` - The sample position where looping ends.
    /// * `one_shot` - Whether playback stops after one pass.
    /// * `base_key` - The MIDI note used as the replacement sample's original pitch.
    pub fn reload(
        &mut self,
        is_drum: bool,
        sample_id: u8,
        loop_start: u32,
        loop_end: u32,
        one_shot: bool,
        base_key: u8,
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
        self.base_key = base_key;

        if self.one_shot {
            self.length = (self.sampler.data[self.sample_id as usize].data.len() / 2) as u32;
        } else {
            self.length = self.loop_start as u32;
        }
    }
}
