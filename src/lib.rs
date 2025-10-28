#![no_std]
//!
//! ```no_run
//! // ⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⡇⢸⣏⡉⡇⣏⡱⡏⢱⡎⢱
//! // ⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠟⠻⠧⠤⠇⠇⠱⠧⠜⠣⠜
//! //
//! // #no-std optimized wave table synthesizer for embedded devices.
//! ```
//!
//! Example usage:
//! ```rust
//! use little_weirdo::synth::{
//!     self,
//!     data::wavetables::{BoxedWavetable, BoxedWavetables},
//! };
//!
//! use std::{
//!     fs,
//!     sync::Arc,
//! };
//!
//! const SAMPLE_RATE: u16 = 44_100; // Audio sample rate in Hz
//!
//! fn main() {
//!
//!     // Create a collection of wavetables and load them from files
//!     let mut wt_on_heap = BoxedWavetables::new();
//!     for id in 0..10 {
//!         let filename = format!("examples/soundbank/soundbank_pure_elektro/src/wav{}.raw", id);
//!         let contents = fs::read(filename).unwrap();
//!         let bytes: &[u8] = &contents;
//!         wt_on_heap.add(BoxedWavetable::new(bytes));
//!     }
//!     // Wrap wavetables in an Arc for thread-safe sharing
//!     let wt = Arc::new(wt_on_heap);
//!     
//!     // Load a synth patch from a JSON file
//!     let patch = serde_json::from_slice(include_bytes!("../examples/patches/bass.json")).unwrap();
//!
//!     // Create a new synthesizer instance with specified parameters
//!     let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE as u16, &patch, Arc::clone(&wt));
//!
//!     // Trigger a note
//!     synth.note_on(60, 100);
//!     
//!     loop {
//!         let _sample:[i16;2] = synth.clock_and_output();
//!         // do something with the sample, stream it to a audio device for example
//!         break;
//!     }
//! }
//! ```

///
/// The wave table synthesizer engine.
///
pub mod synth;
