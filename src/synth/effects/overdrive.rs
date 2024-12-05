//! Overdrive 
use crate::synth::effects::Effect;

pub enum KindOfOverdrive {
    Hard,
    Soft
}


pub struct Overdrive {
   pub threshold: i16,
   pub kind: KindOfOverdrive,
}

impl Overdrive {
    pub fn new( threshold: i16, kind: KindOfOverdrive) -> Self {
        Self {
            threshold,
            kind
        }
    }
}
 

impl Effect for Overdrive {
    fn clock(&mut self, sample: i16) -> i16 {
        match self.kind {
          KindOfOverdrive::Hard => {
            if sample > self.threshold || -sample > self.threshold {
                return self.threshold;
            }
          } 
          KindOfOverdrive::Soft => {
            if sample > self.threshold || -sample > self.threshold {
                if sample > 0 {
                    return (sample-self.threshold) / 4 + self.threshold;
                }
                return (-sample-self.threshold) / 4 - self.threshold;
            }
          } 
        }
        sample 
    }
}