```
 ⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⡇⢸⣏⡉⡇⣏⡱⡏⢱⡎⢱
 ⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠟⠻⠧⠤⠇⠇⠱⠧⠜⠣⠜
```
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/hi-squeaky-things/little-weirdo/rust.yml?branch=main)
![GitHub License](https://img.shields.io/github/license/hi-squeaky-things/little-weirdo)
![Deps.rs Crate Dependencies (specific version)](https://img.shields.io/deps-rs/little_weirdo/0.1.1)
![docs.rs (with version)](https://img.shields.io/docsrs/little_weirdo/0.1.1?style=flat)


A Rust #no-std optimized wave table synthesizer for embedded devices.

> [!CAUTION]
> This project is actively being developed with frequent breaking changes. APIs may shift, features are incomplete, and stability is not guaranteed. Use at your own risk and expect regular updates that might require code adjustments. Have fun!

> [!IMPORTANT]
> **Hi Squeaky Things** can happen at any time. _Little Weirdo_ is ready to squeak, squuuueak, squeeeeeaak, squeaaaaaaaaak!

## Features

- ✅ No-std compatible for embedded devices
- ✅ Lightweight and efficient
- ✅ Wavetable synthesis with customizable patches
- ✅ MIDI support
- ✅ Real-time audio processing
- ✅ Modular architecture for easy extension

## Installation

Add the library to your project:

```bash
cargo add little_weirdo
```

## Quick Start

Here's a basic example to get you started:

``` rust
use little_weirdo::synth::{
    self,
    data::{
        patches::{BoxedPatch, BoxedPatches, Patches},
        wavetables::{BoxedWavetable, BoxedWavetables},
    },
    patch::Patch,
};
use std::{fs, sync::Arc};
const SAMPLE_RATE: u16 = 44_100;
fn main() {
    // Create a collection of wavetables and load them from files.
    let mut wt_on_heap = BoxedWavetables::new();
    for id in 0..10 {
        let filename = format!("examples/soundbank/waveforms/src/wav{}.raw", id);
        let contents = fs::read(filename).unwrap();
        wt_on_heap.add(BoxedWavetable::new(&contents));
    }
    let wavetables = Arc::new(wt_on_heap);
    // Load a synth patch into the boxed patch collection used by the library.
    let mut patches = BoxedPatches::new();
    let patch_data = fs::read("examples/soundbank/patches/orginal/bass.json").unwrap();
    let patch: Patch = serde_json::from_slice(&patch_data).unwrap();
    patches.add(BoxedPatch::new(patch));
    let patches = Arc::new(patches);
    // Create a new synthesizer instance with the patch index and wavetable set.
    let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE, 0, patches, Arc::clone(&wavetables));
    // Trigger a note.
    synth.note_on(60, 100);
    // Get the samples
    for _ in 0..4 {
        let _sample: [i16; 2] = synth.clock_and_output();
    }
}
```

## Examples

- **[Little Weirdo Streaming Audio With MIDI](examples/little_weirdo_streaming_audio_with_midi.rs)**: Run Little Weirdo as a MIDI device while outputting to a local Audio Output.
- **[Little Weirdo Patch Tester](examples/little_weirdo_patch_tester.rs)**: Play with patches for Little Weirdo.
- **[Little Weirdo Generate Soundbank](examples/little_weirdo_generate_soundbanks.rs)**: Generate your own Soundbank using Wavetable samples from AKWF.

## Performance

The performance tests on real embedded hardware can be found here [Little Weirdo ESP32](https://github.com/hi-squeaky-things/little-weirdo-esp32)

## Patches

Patches can be fully programmed in Rust or loaded using JSON (Testing) or Postcard (Embedded Devices). Checkout the [patches](examples/patches/) for examples.

## Documentation

- [API Documentation](https://docs.rs/little_weirdo/0.1.2/little_weirdo/)

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).


