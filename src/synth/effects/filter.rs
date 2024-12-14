//! Lowpass filter 
use crate::synth::effects::Effect;

#[derive(Copy, Clone)]
pub enum KindOfFilter{
    Low,
    High,
    Band,
    Notch,
}


#[derive(Copy, Clone)]
pub struct FilterConfig {
    pub cutoff_frequency: u16,
    pub resonance: u16,   
    pub kind_of_filter: KindOfFilter,
    pub disabled: bool,
}

pub struct Filter {
    config: FilterConfig,
    buf0: i64,
    buf1: i64,
    feedback: u32,
    sample_rate: u16,
}

impl Effect for Filter {
    fn clock(&mut self, sample: i16) -> i16 {
        if self.config.disabled {
            sample
        } else {
            self.filter(sample)
        }
    }
}

impl Filter {
    const FX_SHIFT:u16 = 16;
    const SHIFTED_1: u16 = u16::MAX;
    
    pub fn new(sample_rate: u16, config: FilterConfig) -> Self {
        let mut filter = Self {
            config: config,
            sample_rate,
            buf0: 0,
            buf1: 0,
            feedback: 0,
        };
        filter.prepare_filter();
        filter
    }

    pub fn reload(&mut self, config: FilterConfig) {
        self.config = config;
        self.prepare_filter();
    }

    fn prepare_filter(&mut self) {
        self.feedback = self.config.resonance as u32 + self.ucfxmul( self.config.resonance, Filter::SHIFTED_1 -  self.config.cutoff_frequency);
    }

    fn filter(&mut self, sample: i16) -> i16 {
            let x = self.fxmul(self.feedback as i64, (self.buf0- self.buf1) as i32);
            self.buf0 = self.buf0 + self.fxmul(( (sample as i64 - self.buf0) + x), self.config.cutoff_frequency as i32);
            self.buf1 =  self.buf1 +  self.ifxmul(( self.buf0- self.buf1) as i32, self.config.cutoff_frequency) as i64;
            let out;
            match self.config.kind_of_filter {
                KindOfFilter::Low => out = self.buf1,
                KindOfFilter::High => out = sample as i64 - self.buf0,
                KindOfFilter::Band => out = self.buf0 - self.buf1,
                KindOfFilter::Notch => out = sample as i64 - self.buf0 + self.buf1,
            }
            return out as i16;
    } 

    fn ucfxmul(&mut self, a: u16, b: u16) -> u32 {
        (a as u32* b as u32) >> Filter::FX_SHIFT
    }
    
    fn ifxmul(&mut self, a: i32, b: u16) -> i32 {
        (a * b as i32) >> Filter::FX_SHIFT
    }
    
    fn fxmul(&mut self, a: i64, b: i32) -> i64 {
        (a * b as i64) >> Filter::FX_SHIFT
    }
}
