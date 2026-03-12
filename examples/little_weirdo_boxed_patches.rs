use core::alloc;

use little_weirdo::synth::{data::patches::{BoxedPatch, BoxedPatches, Patches}, patch::{self, Patch}};

fn main() {
    println!("test");
      let patch:Patch = serde_json::from_slice(include_bytes!("soundbank/patches/supersaw_4_oscillators.json")).unwrap();

      let patch_box = BoxedPatch::new(patch);
      let mut patches = BoxedPatches::new();
      patches.add(patch_box);

      let patch =  patches.get_patches_reference(0);

      println!("{:?}", patch.name)

}