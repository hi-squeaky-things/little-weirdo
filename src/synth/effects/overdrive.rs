use crate::synth::effects::Effect;

pub struct Overdrive {
   
}

impl Effect for Overdrive {
    fn clock(&mut self, _sample: i16) -> i16 {
        0
    }
}