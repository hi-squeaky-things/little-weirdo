use std::i16;

use little_weirdo::synth::math;

fn main() {
    for i in 0..3 {
        println!("{:?}", i);
    }
    println!("{:?}", math::percentage(10,400));
}