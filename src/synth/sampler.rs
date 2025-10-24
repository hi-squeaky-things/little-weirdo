
use super::{Clockable};
 
 

 extern crate alloc;
use alloc::{vec::Vec, rc::Rc};

#[derive(Clone)]
pub struct BoxedSample {
    pub data: Vec<i16>
}

impl BoxedSample {
    pub fn new(data: Vec<u8>) -> Self {
        let mut init = Self {
            data: Vec::with_capacity(data.len() / 2),
        };
        for sample_index in 0..(data.len()/2) {
            let b1 = (data[sample_index * 2 + 1] as i16) << 8;
            let b2 = data[sample_index * 2] as i16;
            let sample = b1 | b2;
            init.data.push(sample);
        }    
       init
    }

   
}



pub struct Sampler {
    sampler: Rc<BoxedSample>,
    counter: u32,
    increment: u16,
    speed: u16,
    delay: u16,
    last_sample: i16,
}

impl Clockable for Sampler {
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        self.delay = self.delay + 1;
        if self.delay > self.increment {
            self.delay = 0;
            self.counter = self.counter + self.speed as u32;
            if self.counter >= self.sampler.data.len() as u32 {
                self.counter = 0;
            }
        }
        self.last_sample = self.sampler.data[self.counter as usize];
        self.last_sample
    }
}

impl Sampler {
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
//415, 440, 466, 493
    pub fn change_freq(&mut self, freq: u16) {
        match freq {
            415  => {
                self.speed = 2;
                self.increment = 0;
            },
            440  => {
                self.speed = 2;
                self.increment = 2;
 
            },
            466  => {
                self.speed = 1;
                self.increment = 0;
 
            },
             493  => {
                self.speed = 1;
                self.increment = 2; 
            },
            _ => {

            }
        }
        self.counter = 0;
    }
}