use std::{alloc, mem, sync::Arc, thread};

use little_weirdo::synth::{self, data::wavetables::Wavetable, patch::Patch, patches::Patches, wavetable_oscillator::Waveform};

fn main() {
    let x = Vec::<Wavetable>::with_capacity(10);
   

     let mut patch: Box<Patch> = Box::new(Patch::default());
     let mut synth: Box<synth::Synth> = Box::new(synth::Synth::new(44100, *patch));


   
    synth.note_on(60, 100);
     for n in 0..44100 {
             let output = synth.clock_and_output(); 
           println!("x = {:?}", output);
     }



    


}