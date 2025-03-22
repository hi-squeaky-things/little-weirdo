use crate::synth::patch::Patch;
extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;


pub trait Wavetables  {
    fn get_wavetable_reference(&self, index: u8) -> &[i16];
}

#[derive(Clone)]
pub struct BoxedWavetables {
    data: Vec<BoxedWavetable>
}

impl BoxedWavetables {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(10),
        }
    }

    pub fn add(&mut self, wt: BoxedWavetable) {
        self.data.push(wt);
    }
    
}

#[derive(Clone)]
pub struct BoxedWavetable {
    data: Vec<i16>
}

impl BoxedWavetable {
    pub fn new(data: &[u8]) -> Self {
        let mut init = Self {
            data: Vec::with_capacity(600),
        };
        for sample_index in 0..600 {
            let b1 = (data[sample_index * 2 + 1] as i16) << 8;
            let b2 = data[sample_index * 2] as i16;
            let sample = b1 | b2;
            init.data.push(sample);
        }    
       init
    }

   
}

impl Wavetables for BoxedWavetables {
    fn get_wavetable_reference(&self, index: u8) -> &[i16] {
        self.data[index as usize].data.as_slice()
    }
}



