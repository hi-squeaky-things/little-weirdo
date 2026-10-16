#![no_std]
//! # `The Little Weirdo Digital Audio Synthesizer Library`
//!
//! ```text
//! ⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⡇⢸⣏⡉⡇⣏⡱⡏⢱⡎⢱
//! ⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠟⠻⠧⠤⠇⠇⠱⠧⠜⠣⠜
//! #no-std optimized additive/subtractive/granular/sample based synthesizer and sequencer.
//! ```
//!
//! The Little Weirdo synthesizer library is a high-level Rust library for creating digital audio synthesizers. It provides a flexible and modular architecture for building synthesizers, allowing (squeaky sound) developers/designers to create a wide range of sounds and effects.
//!
//! **[Hi Squeaky Things](https://www.hi-squeaky-things.nl)** can happen at any time™. _Little Weirdo_ is ready to squeak, squuuueak, squeeeeeaak, squeaaaaaaaaak!
//!
//!  See the Little Weirdo in embedded context action, checkout **[Little Squeaky Machine Hardware!](https://github.com/hi-squeaky-things/little-squeaky-machine-hardware)** or
//!  buy the embedded reference hardware @ **[Hi Squeaky Things](https://www.hi-squeaky-things.nl)** to support the development of this library.
//!
//!
//! ## Features
//! - `no_std` library optimized for embedded devices
//! - Lightweight and efficient (no floating point calculations)
//! - Waveform (table) based additive/subtractive and granular (sort-of) synthesizer.
//! - Sample (sound-font-a-like) based synthesizer
//! - Customizable patches
//! - Real-time audio processing and effects (distortion/overdrive, filters, echo/delay, bit-crunch, lfo, ring-modulation)
//! - Modular architecture for easy extension
//!
//! The library is designed to be highly customizable, allowing developers to create their own synthesizers and effects using the provided components.
//!
//! ## Performance
//!
//! The performance tests on real embedded hardware can be found here [Little Weirdo performance tests on a ESP32/ESP32S3](https://github.com/hi-squeaky-things/little-weirdo-esp32)
//!
//! ## Examples
//!
//! Example usage of the additive/subtractive/granular based synthesizer components:
//! ```rust
//! use little_weirdo::synth::{
//!     self,
//!     data::{
//!         patches::{BoxedPatch, BoxedPatches, Patches},
//!         waveforms::{BoxedWaveform, BoxedWaveforms},
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
//!     let mut oscillator_waveforms = BoxedWaveforms::new();
//!     for id in 0..10 {
//!         let filename = format!("examples/soundbank/synth/waveforms/src/{:03}_sample.raw", id);
//!         let contents = fs::read(filename).unwrap();
//!         oscillator_waveforms.add(BoxedWaveform::new(&contents));
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
//!     println!("___Hi Squeaky Things! can happen at any time ™___");
//! }
//! ```
//!
//! Example usage of the sample (sound-font-a-like) based synthesizer components:
//! ```rust
//! use little_weirdo::wavetable::{
//!     self,
//!     sample_voice::{BoxedSample, BoxedSamples},
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
//!         let filename = format!("examples/soundbank/wavetable/samples/src/{:03}_sample.raw", id);
//!         let contents = fs::read(filename).unwrap();
//!         samples.add(BoxedSample::new(contents));
//!     }
//!
//!     let samples_on_heap = Arc::new(samples);
//!     
//!     // Load a sampler patch into the boxed patch collection used by the library.
//!     let mut patches = BoxedSamplerPatches::new();
//!     let patch_data = fs::read("examples/soundbank/wavetable/patches/original/01_piano.json").unwrap();
//!     let patch: Patch = serde_json::from_slice(&patch_data).unwrap();
//!     patches.add(BoxedSamplerPatch::new(patch));
//!     let patches = Arc::new(patches);
//!
//!     // Create a new sampler instance with the patch index and sample set.
//!     let mut sampler: wavetable::WavetableSynth = wavetable::WavetableSynth::new(SAMPLE_RATE, 0, patches, Arc::clone(&samples_on_heap));
//!
//!     // Trigger a note.
//!     sampler.note_on(60, 100);
//!
//!     // Get the samples
//!     for _ in 0..4 {
//!         let _sample: [i16; 2] = sampler.clock_and_output();
//!     }
//!     println!("___Hi Squeaky Things! can happen at any time ™___");
//! }
//! ```

///
/// The Little Weirdo additive/subtractive and granular (sort-of) synthesizer.
///
pub mod synth;

///
/// The Little Weirdo wavetable (sound-font-a-like) based synthesizer.
///
pub mod wavetable;

///
/// The Little Weirdo STEP-SEQUENCER
///
pub mod sequencer;

///
/// The Little Weirdo EFFECTS (delay/echo, filters, overdrive/distortion, bit-crunch)
///
pub mod effects;

///
/// Math helper functions optimized for embedded devices (no floating point operations)
///
pub mod math;

// TODO: Update the waveform data to use 1024 samples per waveform for better audio quality and more accurate representation of the waveforms. This will allow for smoother sound generation and improved performance in the synthesizer.
