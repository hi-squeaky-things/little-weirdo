```
 ⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⡇⢸⣏⡉⡇⣏⡱⡏⢱⡎⢱
 ⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠟⠻⠧⠤⠇⠇⠱⠧⠜⠣⠜
```
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/hi-squeaky-things/little-weirdo/rust.yml?branch=main)
![GitHub License](https://img.shields.io/github/license/hi-squeaky-things/little-weirdo)
![Deps.rs Crate Dependencies (specific version)](https://img.shields.io/deps-rs/little_weirdo/0.1.3)
![docs.rs (with version)](https://img.shields.io/docsrs/little_weirdo/0.1.3?style=flat)

A Rust #no-std optimized wave table synthesizer for embedded devices.

Our first project is launching soon using the LttL Weirdo: our LttL Squeaky Machine! Put yourself on the list to get notified when we are ready to ship. 
We are iteration on revision 3.0. Soon to be released.

[Join the waitinglist](https://www.hi-squeaky-things.nl/waiting-list)

<img src="https://www.hi-squeaky-things.nl/web/image/1400-5c54f567/Jouw%20alineatekst%20%281%29.webp" width="300">

[See Little Squeaky Machine Hardware!](https://github.com/hi-squeaky-things/little-squeaky-machine-hardware)

> [!CAUTION]
> This project is actively being developed with frequent breaking changes. APIs may shift, features are incomplete, and stability is not guaranteed. Use at your own risk and expect regular updates that might require code adjustments. Have fun!

> [!IMPORTANT]
> **Hi Squeaky Things** can happen at any time. _Little Weirdo_ is ready to squeak, squuuueak, squeeeeeaak, squeaaaaaaaaak!

## Features

- ✅ `no_std` library optimized for embedded devices
- ✅ Lightweight and efficient (no floating point calculations)
- ✅ WAVEFORM (table) based additive/subtractive and granular (sort-of) synthesizer.
- ✅ SAMPLER (sound-font-a-like) based synthesizer
- ✅ Customizable patches
- ✅ Real-time audio processing (distortion/overdrive, filter, delay, bit-crunch, lfo, ring-modulation)
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
        patches::{BoxedPatch, BoxedPatches},
        waveforms::{BoxedWaveform, BoxedWaveforms},
    },
};

use std::{fs, sync::Arc};

const SAMPLE_RATE: u16 = 44_100;

fn main() {
    // Create a collection of waveforms and load them from files.
    let mut oscillator_waveforms = BoxedWaveforms::new();
    for id in 0..10 {
        let filename = format!("examples/soundbank/synth/waveforms/src/{:03}_sample.raw", id);
        let contents = fs::read(filename).unwrap();
        oscillator_waveforms.add(BoxedWaveform::new(&contents));
    }
    let oscillator_waveforms_on_heap = Arc::new(oscillator_waveforms);

    // Load a synth patch into the boxed patch collection used by the library.
    let mut patches = BoxedPatches::new();
    let patch_data = fs::read("examples/soundbank/synth/patches/original/bass.json").unwrap();
    let patch = serde_json::from_slice(&patch_data).unwrap();
    patches.add(BoxedPatch::new(patch));
    let patches = Arc::new(patches);

    // Create a new synthesizer instance with the patch index and wavetable set.
    let mut synth: synth::Synth = synth::Synth::new(
        SAMPLE_RATE,
        0,
        patches,
        Arc::clone(&oscillator_waveforms_on_heap),
    );

    // Trigger a note.
    synth.note_on(60, 100);

    for _ in 0..400 {
        let sample: [i16; 2] = synth.clock_and_output();
        println!("sample = {:?}", sample);
    }
}
```

## Examples

- **[Little Weirdo Streaming Audio With MIDI](examples/little_weirdo_synth_streaming_audio_with_midi.rs)**: Run Little Weirdo as a MIDI device while outputting to a local Audio Output.
- **[Little Weirdo Sampler Streaming Audio With MIDI](examples/little_weirdo_sampler_streaming_audio_with_midi.rs)**: Run Little Weirdo Sampler as a MIDI device while outputting to a local Audio Output.
- **[Little Weirdo Patch Tester](examples/little_weirdo_patch_tester.rs)**: Play with patches for Little Weirdo.
- **[Little Weirdo Generate Soundbank](examples/little_weirdo_generate_soundbanks.rs)**: Generate your own Soundbank using Wavetable samples from AKWF.

## Performance

The performance tests on real embedded hardware can be found here [Little Weirdo ESP32](https://github.com/hi-squeaky-things/little-weirdo-esp32)

## Patches

Patches can be fully programmed in Rust or loaded using JSON (Testing) or Postcard (Embedded Devices). 
- [synth patches](examples/soundbank/synth/patches/original/) for examples.
- [sampler patches](examples/soundbank/sampler/patches/) for examples.

## Documentation

- [API Documentation](https://docs.rs/little_weirdo/0.1.3/little_weirdo/)

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).


