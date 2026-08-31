#![no_std]
//!
//! ```no_run
//! // ⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⡇⢸⣏⡉⡇⣏⡱⡏⢱⡎⢱
//! // ⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠟⠻⠧⠤⠇⠇⠱⠧⠜⠣⠜
//! //
//! // #no-std optimized wave table synthesizer, sampler and sequencer for embedded devices.
//! ```
//!
//! Example usage:
//! ```rust
//! use little_weirdo::synth::{
//!     self,
//!     data::{
//!         patches::{BoxedPatch, BoxedPatches, Patches},
//!         wavetables::{BoxedWavetable, BoxedWavetables},
//!     },
//!     patch::Patch,
//! };
//!
//! use std::{fs, sync::Arc};
//!
//! const SAMPLE_RATE: u16 = 44_100;
//!
//! fn main() {
//!     // Create a collection of waveforms and load them from files.
//!     let mut oscillator_waveforms = BoxedWavetables::new();
//!     for id in 0..10 {
//!         let filename = format!("examples/soundbank/synth/src/{:03}_sample.raw", id);
//!         let contents = fs::read(filename).unwrap();
//!         oscillator_waveforms.add(BoxedWavetable::new(&contents));
//!     }
//!     let oscillator_waveforms_on_heap = Arc::new(oscillator_waveforms);
//!
//!     // Load a synth patch into the boxed patch collection used by the library.
//!     let mut patches = BoxedPatches::new();
//!     let patch_data = fs::read("examples/soundbank/synth/patches/original/bass.json").unwrap();
//!     let patch: Patch = serde_json::from_slice(&patch_data).unwrap();
//!     patches.add(BoxedPatch::new(patch));
//!     let patches = Arc::new(patches);
//!
//!     // Create a new synthesizer instance with the patch index and wavetable set.
//!     let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE, 0, patches, Arc::clone(&oscillator_waveforms_on_heap));
//!
//!     // Trigger a note.
//!     synth.note_on(60, 100);
//!
//!     // Get the samples
//!     for _ in 0..4 {
//!         let _sample: [i16; 2] = synth.clock_and_output();
//!     }
//! }
//! ```
//!
//! Example usage with sampler:
//! ```rust
//! use little_weirdo::sampler::{
//!     self,
//!     audio_sampler::{BoxedSample, BoxedSamples},
//!     data::patches::{BoxedSamplerPatch, BoxedSamplerPatches},
//!     patch::Patch,
//! };
//!
//! use std::{fs, sync::Arc};
//!
//! const SAMPLE_RATE: u16 = 44_100;
//!
//! fn main() {
//!     // Create a collection of samples and load them from files.
//!     let mut samples = BoxedSamples::new();
//!     for id in 0..15 {
//!         let filename = format!("examples/soundbank/sampler/src/{:03}_sample.raw", id);
//!         let contents = fs::read(filename).unwrap();
//!         samples.add(BoxedSample::new(contents));
//!     }
//!
//!     let samples_on_heap = Arc::new(samples);
//!     
//!     // Load a sampler patch into the boxed patch collection used by the library.
//!     let mut patches = BoxedSamplerPatches::new();
//!     let patch_data = fs::read("examples/soundbank/sampler/patches/01_piano.json").unwrap();
//!     let patch: Patch = serde_json::from_slice(&patch_data).unwrap();
//!     patches.add(BoxedSamplerPatch::new(patch));
//!     let patches = Arc::new(patches);
//!
//!     // Create a new sampler instance with the patch index and sample set.
//!     let mut sampler: sampler::Sampler = sampler::Sampler::new(SAMPLE_RATE, 0, patches, Arc::clone(&samples_on_heap));
//!
//!     // Trigger a note.
//!     sampler.note_on(60, 100);
//!
//!     // Get the samples
//!     for _ in 0..4 {
//!         let _sample: [i16; 2] = sampler.clock_and_output();
//!     }
//! }
//! ```

///
/// The Little Weirdo waveform (table) based subtractive synthesizer.
///
pub mod synth;

///
/// The Little Weirdo sampler.
///
pub mod sampler;

///
/// The Little Weirdo step-sequencer
///
pub mod sequencer;

///
/// The Little Weirdo effects
///
pub mod effects;

///
/// Math helper functions optimized for embedded devices with no FPU
///
pub mod math;

// TODO: Update the waveform data to use 1024 samples per waveform for better audio quality and more accurate representation of the waveforms. This will allow for smoother sound generation and improved performance in the synthesizer.
// TODO: Implement the sampler based additive synthesizer
