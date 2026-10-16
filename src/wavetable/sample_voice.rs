extern crate alloc;
use alloc::{sync::Arc, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::synth::{data::frequencies::MIDI2FREQ, Clockable};

const SAMPLE_POSITION_SCALE: u64 = 1 << 32;

// Configuration for standard waveform oscillator
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct SampleVoiceConfig {
    sample_id: u8,
    is_drums: bool,
    loop_start: u32,
    loop_end: u32,
    one_shot: bool,
    base_key: u8,
}

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
        Self { data }
    }
}

/// A voice that plays back an audio sample at variable speeds.
#[allow(dead_code)]
pub struct SampleVoice {
    /// Reference to the underlying audio sample data.
    sampler: Arc<BoxedSamples>,

    loop_is_started: bool,

    sample_id: u8,
    /// Current position in the audio sample data.
    counter: u64,
    /// Speed increment for advancing through the sample data.
    increment: u64,
    open: bool,
    length: u32,
    config: SampleVoiceConfig,
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
        if (self.counter / SAMPLE_POSITION_SCALE) as u32 >= self.length {
            if !self.loop_is_started && !self.config.one_shot {
                self.loop_is_started = true;
                self.length = self.config.loop_end - self.config.loop_start;
            }
            if self.config.one_shot {
                self.open = false;
            }

            self.counter = 0;

            if self.loop_is_started && !self.config.one_shot {
                let pointer = self.config.loop_start as usize * 2;
                let b1 = (self.sampler.data[self.sample_id as usize].data[pointer + 1] as i16) << 8;
                let b2 = self.sampler.data[self.sample_id as usize].data[pointer] as i16;
                return b1 | b2;
            }
            return 0;
        }
        // Calculate initial pointer position (samples are 16-bit, so multiply by 2)
        let mut pointer = (self.counter / SAMPLE_POSITION_SCALE) as usize * 2;

        if self.loop_is_started {
            // Add loop start offset
            pointer += self.config.loop_start as usize * 2;

            // Check if we've reached the loop end
            let loop_end_pos = (self.config.loop_end * 2) as usize;
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
        if !pointer.is_multiple_of(2) && pointer > 0 {
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
    pub fn new(config: SampleVoiceConfig, sampler: Arc<BoxedSamples>) -> Self {
        let mut audio_sampler = SampleVoice {
            config,
            sampler,
            sample_id: config.sample_id,
            counter: 0,
            increment: SAMPLE_POSITION_SCALE,
            length: 0,
            loop_is_started: false,
            open: false,
        };
        if audio_sampler.config.one_shot {
            audio_sampler.length = (audio_sampler.sampler.data
                [audio_sampler.config.sample_id as usize]
                .data
                .len()
                / 2) as u32;
        } else {
            audio_sampler.length = audio_sampler.config.loop_start;
        }

        if audio_sampler.config.loop_end == 0 {
            audio_sampler.config.loop_end = audio_sampler.length;
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
        if self.config.is_drums {
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
        self.counter = 0;
        self.open = true;
        self.loop_is_started = false;
        if self.config.one_shot {
            self.length = (self.sampler.data[self.sample_id as usize].data.len() / 2) as u32;
        } else {
            self.length = self.config.loop_start;
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
    /// The increment is calculated as `freq / base_frequency`, where both
    /// frequencies use the MIDI table's fifths-of-a-hertz units. The playback
    /// position is reset before the new frequency takes effect.
    ///
    /// # Arguments
    ///
    /// * `freq` - The target frequency in fifths of a hertz.
    pub fn change_freq(&mut self, freq: u16) {
        self.counter = 0;
        let base_frequency = MIDI2FREQ[self.config.base_key as usize] as u64;
        self.increment =
            (freq as u64 * SAMPLE_POSITION_SCALE + base_frequency / 2) / base_frequency;
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
    pub fn reload(&mut self, config: SampleVoiceConfig) {
        self.open = false;
        self.counter = 0;
        self.config = config;
        self.increment = SAMPLE_POSITION_SCALE;
        self.loop_is_started = false;

        if self.config.one_shot {
            self.length = (self.sampler.data[self.sample_id as usize].data.len() / 2) as u32;
        } else {
            self.length = self.config.loop_start;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn sample_voice_plays_at_the_pitch_ratio_from_midi_frequencies() {
        let mut samples = BoxedSamples::new();
        samples.add(BoxedSample::new(vec![1, 0, 2, 0, 3, 0, 4, 0]));
        let config = SampleVoiceConfig {
            sample_id: 0,
            is_drums: false,
            loop_start: 4,
            loop_end: 0,
            one_shot: true,
            base_key: 69,
        };
        let mut voice = SampleVoice::new(config, Arc::new(samples));

        voice.set_note(81);

        assert_eq!(voice.increment, 2 * SAMPLE_POSITION_SCALE);
        voice.open_gate();
        assert_eq!(voice.clock(None), 3);
    }
}
