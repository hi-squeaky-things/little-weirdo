extern crate alloc;
use alloc::vec::Vec;

// Trait defining the interface for accessing waveforms by index
pub trait Waveforms {
    // Returns a reference to the waveform data at the specified index
    fn get_waveform_reference(&self, index: u8) -> &[i16];
}

// Struct that holds multiple boxed Waveforms
#[derive(Clone)]
pub struct BoxedWaveforms {
    data: Vec<BoxedWaveform>,
}

impl Default for BoxedWaveforms {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxedWaveforms {
    // Creates a new empty BoxedWaveforms instance with capacity for 10 waveforms
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(10),
        }
    }

    // Adds a new wavetable to the collection
    pub fn add(&mut self, wt: BoxedWaveform) {
        self.data.push(wt);
    }
}

// Struct representing a single wavetable containing audio samples
#[derive(Clone)]
pub struct BoxedWaveform {
    data: Vec<i16>,
}

impl BoxedWaveform {
    // Creates a new waveform from raw byte data
    // Each sample is assumed to be 2 bytes (16-bit) in big-endian format
    pub fn new(data: &[u8]) -> Self {
        let mut init = Self {
            data: Vec::with_capacity(600),
        };
        for sample_index in 0..600 {
            // Extract two bytes and combine them into a 16-bit signed integer
            // The high byte is shifted left by 8 bits and combined with the low byte
            let b1 = (data[sample_index * 2 + 1] as i16) << 8;
            let b2 = data[sample_index * 2] as i16;
            let sample = b1 | b2;
            init.data.push(sample);
        }
        init
    }
}

// Implementation of the Waveforms trait for BoxedWaveforms
// Allows retrieving a specific waveform by its index
impl Waveforms for BoxedWaveforms {
    fn get_waveform_reference(&self, index: u8) -> &[i16] {
        self.data[index as usize].data.as_slice()
    }
}
