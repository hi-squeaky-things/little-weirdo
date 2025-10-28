use std::{fs, sync::Arc};

use criterion::{criterion_group, criterion_main, Criterion};
use little_weirdo::synth::{
    self,
    data::wavetables::{BoxedWavetable, BoxedWavetables},
};

const SAMPLE_RATE: u16 = 44_100; // Audio sample rate in Hz

pub fn criterion_benchmark(c: &mut Criterion) {
    // Initialize wavetables storage on heap
    let mut wt_on_heap = BoxedWavetables::new();

    // Load 10 wavetables from files
    for id in 0..10 {
        let filename = format!(
            "examples/soundbank/soundbank_pure_elektro/src/wav{}.raw",
            id
        );
        let contents = fs::read(filename).unwrap(); // Read file contents
        let bytes: &[u8] = &contents; // Convert to byte slice
        wt_on_heap.add(BoxedWavetable::new(bytes)); // Add to wavetables collection
    }

    // Create an Arc (thread-safe reference) to the wavetables
    let wt = Arc::new(wt_on_heap);

    // Load a synth patch from a JSON file
    let patch = serde_json::from_slice(include_bytes!("../examples/patches/bass.json")).unwrap();

    // Create a new synthesizer instance with specified parameters
    let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE as u16, &patch, Arc::clone(&wt));
    synth.note_on(60, 100);
    c.bench_function("Little Weirdo Synth Clock and Output", |b| {
        b.iter(|| synth.clock_and_output())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
