use std::mem;

use little_weirdo::synth::data::wavetables::{SoundBank, Wavetable};
use little_weirdo_soundbanks::soundbanks::SOUND_BANK_WILD_FRUIT;

fn main() {
    let wt:Box<SoundBank> = Box::new(SOUND_BANK_WILD_FRUIT);
    let wtt = wt.wavetables;
    println!("size {:?}", mem::size_of_val(&wt));
}
