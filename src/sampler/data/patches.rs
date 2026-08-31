extern crate alloc;
use alloc::vec::Vec;
use alloc::boxed::Box;

use crate::sampler::patch::Patch;

pub trait Patches {
    fn get_patches_reference(&self, index: u8) -> &Patch;
}

#[derive(Clone)]
pub struct BoxedSamplerPatches{
    data: Vec<BoxedSamplerPatch>,
}

impl Default for BoxedSamplerPatches {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxedSamplerPatches {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(10),
        }
    }

    // Adds a new patch to the collection
    pub fn add(&mut self, wt: BoxedSamplerPatch) {
        self.data.push(wt);
    }
}

// Struct representing a single patch containing audio samples
#[derive(Clone)]
pub struct BoxedSamplerPatch {
    data: Box<Patch>,
}

impl BoxedSamplerPatch {
    // Creates a new patch from raw byte data
    // Each sample is assumed to be 2 bytes (16-bit) in big-endian format
    pub fn new(patch: Patch) -> Self {
         Self {
            data: Box::new(patch)
         }
    }
}

// Implementation of the patchs trait for BoxedSamplerpatchs
// Allows retrieving a specific patch by its index
impl Patches for BoxedSamplerPatches {
    fn get_patches_reference(&self, index: u8) -> &Patch {
        &self.data[index as usize].data
    }
}
