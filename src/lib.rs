#![no_std]
//! 
//! ```no_run 
//! //       __   ____  ____  __      _  _  ____  __  ____  ____   __  
//! //      (  ) (_  _)(_  _)(  )    / )( \(  __)(  )(  _ \(    \ /  \ 
//! //      / (_/\ )(    )(  / (_/\  \ /\ / ) _)  )(  )   / ) D ((  O )
//! //      \____/(__)  (__) \____/  (_/\_)(____)(__)(__\_)(____/ \__/ 
//! //
//! //       #no-std optimized wave table synthesizer for embedded devices.
//! ```
//! 
//! Example usage:
//! ```
//! 
//! use little_weirdo::synth::patch::Patch;
//! use little_weirdo::synth::Synth;
//! use little_weirdo_soundbanks::soundbanks::SOUND_BANK_PURE_ELEKTRO;
//! use little_weirdo_soundbanks::patches::Patches;
//! 
//! fn main() {
//!     // Choose a predefined patch (or create your own).
//!     let patch: Patch = Patches::get_patch(Patches::BassGuitar);
//!     // Create an instance of LttL Weirdo.
//!     let mut synth: Synth = Synth::new(44100, patch, &SOUND_BANK_PURE_ELEKTRO);
//!     // Start playing a note
//!     synth.note_on(0x50, 0xFF);
//!     // Clock LttL Weirdo in a loop.
//!     let sample = synth.clock_and_output();
//! }
//! ```


///
/// The wave table synthesizer engine.
/// 
pub mod synth;
