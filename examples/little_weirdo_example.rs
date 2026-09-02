use little_weirdo::synth::{
    self,
    data::{
        patches::{BoxedPatch, BoxedPatches},
        waveforms::{BoxedWaveform, BoxedWaveforms},
    },
};

use std::{fs, sync::Arc};

const SAMPLE_RATE: u16 = 44_100;

fn main() {
    // Create a collection of waveforms and load them from files.
    let mut oscillator_waveforms = BoxedWaveforms::new();
    for id in 0..10 {
        let filename = format!(
            "examples/soundbank/synth/waveforms/src/{:03}_sample.raw",
            id
        );
        let contents = fs::read(filename).unwrap();
        oscillator_waveforms.add(BoxedWaveform::new(&contents));
    }
    let oscillator_waveforms_on_heap = Arc::new(oscillator_waveforms);

    // Load a synth patch into the boxed patch collection used by the library.
    let mut patches = BoxedPatches::new();
    let patch_data = fs::read("examples/soundbank/synth/patches/original/bass.json").unwrap();
    let patch = serde_json::from_slice(&patch_data).unwrap();
    patches.add(BoxedPatch::new(patch));
    let patches = Arc::new(patches);

    // Create a new synthesizer instance with the patch index and wavetable set.
    let mut synth: synth::Synth = synth::Synth::new(
        SAMPLE_RATE,
        0,
        patches,
        Arc::clone(&oscillator_waveforms_on_heap),
    );

    // Trigger a note.
    synth.note_on(60, 100);

    for _ in 0..400 {
        let sample: [i16; 2] = synth.clock_and_output();
        println!("sample = {:?}", sample);
    }
}
