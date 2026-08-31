extern crate alloc;
use crate::synth::patch::Patch;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub trait Patches {
    fn get_patches_reference(&self, index: u8) -> &Patch;
}

#[derive(Clone)]
pub struct BoxedPatches {
    data: Vec<BoxedPatch>,
}

impl Default for BoxedPatches {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxedPatches {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(10),
        }
    }

    // Adds a new patch to the collection
    pub fn add(&mut self, wt: BoxedPatch) {
        self.data.push(wt);
    }

    pub fn size(&mut self) -> usize {
        self.data.len()
    }
}

// Struct representing a single patch containing audio samples
#[derive(Clone)]
pub struct BoxedPatch {
    data: Box<Patch>,
}

impl BoxedPatch {
    // Creates a new patch from raw byte data
    // Each sample is assumed to be 2 bytes (16-bit) in big-endian format
    pub fn new(patch: Patch) -> Self {
        Self {
            data: Box::new(patch),
        }
    }
}

// Implementation of the patchs trait for Boxedpatchs
// Allows retrieving a specific patch by its index
impl Patches for BoxedPatches {
    fn get_patches_reference(&self, index: u8) -> &Patch {
        &self.data[index as usize].data
    }
}
