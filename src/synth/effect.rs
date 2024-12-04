pub trait Effect {
    fn clock(&mut self, sample: i16) -> i16;
}