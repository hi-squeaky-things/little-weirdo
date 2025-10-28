use super::Clockable;

extern crate alloc;
use alloc::{rc::Rc, vec::Vec};

/// A boxed sample containing audio data as a vector of 16-bit signed integers.
/// TODO: Work in Progress, not enabled yet.
#[derive(Clone)]
pub struct BoxedSample {
    /// The actual audio sample data.
    pub data: Vec<i16>,
}

impl BoxedSample {
    /// Creates a new `BoxedSample` from a vector of 8-bit unsigned integers.
    /// 
    /// This constructor converts 16-bit little-endian samples from the input data
    /// into a vector of 16-bit signed integers for internal processing.
    pub fn new(data: Vec<u8>) -> Self {
        let mut init = Self {
            data: Vec::with_capacity(data.len() / 2),
        };
        for sample_index in 0..(data.len() / 2) {
            let b1 = (data[sample_index * 2 + 1] as i16) << 8;
            let b2 = data[sample_index * 2] as i16;
            let sample = b1 | b2;
            init.data.push(sample);
        }
        init
    }
}

/// A sampler that plays back audio samples at different speeds.
pub struct Sampler {
    /// Reference to the underlying audio sample data.
    sampler: Rc<BoxedSample>,
    /// Current position in the audio sample data.
    counter: u32,
    /// Speed increment for advancing through the sample data.
    increment: u16,
    /// Playback speed multiplier.
    speed: u16,
    /// Delay counter used for timing control.
    delay: u16,
    /// Last played sample value.
    last_sample: i16,
}

impl Clockable for Sampler {
    /// Processes one clock cycle of the sampler.
    /// 
    /// Advances the playback position based on the configured speed and increment,
    /// and returns the current sample value.
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        self.delay += 1;
        if self.delay > self.increment {
            self.delay = 0;
            self.counter += self.speed as u32;
            if self.counter >= self.sampler.data.len() as u32 {
                self.counter = 0;
            }
        }
        self.last_sample = self.sampler.data[self.counter as usize];
        self.last_sample
    }
}

impl Sampler {
    /// Creates a new sampler instance with the given sample rate and audio data.
    pub fn new(_sample_rate: u16, sampler: Rc<BoxedSample>) -> Self {
        Sampler {
            sampler,
            counter: 0,
            increment: 0,
            speed: 1,
            delay: 0,
            last_sample: 0,
        }
    }
    
    /// Changes the playback frequency by adjusting speed and increment values.
    /// 
    /// This method maps specific frequencies to corresponding speed and increment settings
    /// to achieve desired pitch variations.
    pub fn change_freq(&mut self, freq: u16) {
        match freq {
            415 => {
                self.speed = 2;
                self.increment = 0;
            }
            440 => {
                self.speed = 2;
                self.increment = 2;
            }
            466 => {
                self.speed = 1;
                self.increment = 0;
            }
            493 => {
                self.speed = 1;
                self.increment = 2;
            }
            _ => {}
        }
        self.counter = 0;
    }
}