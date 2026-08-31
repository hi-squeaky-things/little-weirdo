use little_weirdo::synth::{
    self,
    data::{
        patches::{BoxedPatch, BoxedPatches},
        wavetables::{BoxedWavetable, BoxedWavetables},
    },
};

use std::{fs, sync::Arc};

const SAMPLE_RATE: u16 = 44_100;

fn main() {
    // Create a collection of wavetables and load them from files.
    let mut wt_on_heap = BoxedWavetables::new();
    for id in 0..10 {
        let filename = format!("examples/soundbank/waveforms/src/{}_sample.raw", id);
        let contents = fs::read(filename).unwrap();
        wt_on_heap.add(BoxedWavetable::new(&contents));
    }
    let wavetables = Arc::new(wt_on_heap);

    // Load a synth patch into the boxed patch collection used by the library.
    let mut patches = BoxedPatches::new();
    let patch_data = fs::read("examples/soundbank/patches/original/bass.json").unwrap();
    let patch = serde_json::from_slice(&patch_data).unwrap();
    patches.add(BoxedPatch::new(patch));
    let patches = Arc::new(patches);

    // Create a new synthesizer instance with the patch index and wavetable set.
    let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE, 0, patches, Arc::clone(&wavetables));

    // Trigger a note.
    synth.note_on(60, 100);

    for _ in 0..4 {
        let _sample: [i16; 2] = synth.clock_and_output();
    }
}
