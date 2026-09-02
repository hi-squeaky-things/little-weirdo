use std::{fs, sync::Arc};

use criterion::{criterion_group, criterion_main, Criterion};
use little_weirdo::synth::{
    self,
    data::waveforms::{BoxedWaveform, BoxedWaveforms},
};

const SAMPLE_RATE: u16 = 44_100; // Audio sample rate in Hz

pub fn criterion_benchmark(c: &mut Criterion) {
    // Initialize wavetables storage on heap
    let mut wt_on_heap = BoxedWaveforms::new();

    // Load 10 wavetables from files
    for id in 0..10 {
        let filename = format!("examples/soundbank/synth/waveforms/src/{:03}_sample.raw", id);
        let contents = fs::read(filename).unwrap(); // Read file contents
        let bytes: &[u8] = &contents; // Convert to byte slice
        wt_on_heap.add(BoxedWaveform::new(bytes)); // Add to wavetables collection
    }

    // Create an Arc (thread-safe reference) to the wavetables
    let wt = Arc::new(wt_on_heap);

    // Load a synth patch from a JSON file.
    let patch_data = fs::read("examples/soundbank/synth/patches/original/bass.json").unwrap();
    let patch = serde_json::from_slice(&patch_data).unwrap();

    // Create a new synthesizer instance with specified parameters.
    let mut patches = little_weirdo::synth::data::patches::BoxedPatches::new();
    patches.add(little_weirdo::synth::data::patches::BoxedPatch::new(patch));
    let patches = Arc::new(patches);
    let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE, 0, patches, Arc::clone(&wt));
    synth.note_on(60, 100);
    c.bench_function("Little Weirdo Synth Clock and Output", |b| {
        b.iter(|| synth.clock_and_output())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
