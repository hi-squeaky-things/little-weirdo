extern crate alloc;
use alloc::vec::Vec;

// Trait defining the interface for accessing wavetables by index
pub trait Wavetables {
    // Returns a reference to the wavetable data at the specified index
    fn get_wavetable_reference(&self, index: u8) -> &[i16];
}

// Struct that holds multiple boxed wavetables
#[derive(Clone)]
pub struct BoxedWavetables {
    data: Vec<BoxedWavetable>,
}

impl BoxedWavetables {
    // Creates a new empty BoxedWavetables instance with capacity for 10 wavetables
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(10),
        }
    }

    // Adds a new wavetable to the collection
    pub fn add(&mut self, wt: BoxedWavetable) {
        self.data.push(wt);
    }
}

// Struct representing a single wavetable containing audio samples
#[derive(Clone)]
pub struct BoxedWavetable {
    data: Vec<i16>,
}

impl BoxedWavetable {
    // Creates a new wavetable from raw byte data
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

// Implementation of the Wavetables trait for BoxedWavetables
// Allows retrieving a specific wavetable by its index
impl Wavetables for BoxedWavetables {
    fn get_wavetable_reference(&self, index: u8) -> &[i16] {
        self.data[index as usize].data.as_slice()
    }
}
