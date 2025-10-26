# Little Weirdo Patches

This repository contains example patches for the Little Weirdo synthesizer, available in two formats:

- **JSON** (.json) - For local development and testing
- **POSTCARD** (.lpw) - For embedded systems and constrained environments

## JSON

This patches are for testing the Little Weirdo synth on your local machine; not indented for constraint based environment like Embedded systems.

## POSTCARD

These patches are for constraint based environment like Embedded systems. We are using the `postcard` Rust library for the serialization an deserialization of those patches.

```rust
use little_weirdo::synth::patch::Patch;
use postcard;

let patch_bytes: &[u8] = include_bytes!("patch.lpw");
let patch: Patch = postcard::from_bytes(patch_bytes).unwrap();
```
