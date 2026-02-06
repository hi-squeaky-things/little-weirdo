
pub mod overdrive;
pub mod filter;
pub mod bitcrunch;


pub trait Effect {
    fn clock(&mut self, sample: i16) -> i16;
}
