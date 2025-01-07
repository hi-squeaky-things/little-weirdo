//! Overdrive 
use serde::{Deserialize, Serialize};

use crate::synth::{effects::Effect};


#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum KindOfOverdrive {
    Hard,
    Soft,
    Softer,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct OverdriveConfiguration {
    pub threshold: i16,
    pub kind: KindOfOverdrive,
    pub enabled: bool,
}

pub struct Overdrive {
   pub config: OverdriveConfiguration
}



impl Overdrive {
    pub fn new( config: OverdriveConfiguration) -> Self {
        Self {
            config,
        }
    }

    pub fn reload(&mut self, config: OverdriveConfiguration) {
        self.config = config;
    }
}
 
impl Effect for Overdrive {
    fn clock(&mut self, sample: i16) -> i16 {
        if !self.config.enabled {
            return sample
        }
 

        match self.config.kind {
          KindOfOverdrive::Hard => {
            if sample > self.config.threshold || -sample > self.config.threshold {
                if sample > 0 {
                    return self.config.threshold;
                } 
                return -self.config.threshold;
            }
          } 
          KindOfOverdrive::Soft => {
            if sample > self.config.threshold || -sample > self.config.threshold {
                if sample > 0 {
                    return (sample-self.config.threshold) / 2 + self.config.threshold;
                }
                return (sample + self.config.threshold) / 2 - self.config.threshold;
            }
          } 
          KindOfOverdrive::Softer => {
            if sample > self.config.threshold || -sample > self.config.threshold {
                if sample > 0 {
                    return (sample-self.config.threshold) / 8 + self.config.threshold;
                }
                return (sample + self.config.threshold) / 8 - self.config.threshold;
            }
          } 
        }
        sample 
    }
}