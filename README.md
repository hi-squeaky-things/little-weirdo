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

## How to use it

Get the library!
```
$ cargo add little_weirdo
```

Start using it in your own code:

``` rust
use little_weirdo::synth::{
    self,
    data::wavetables::{BoxedWavetable, BoxedWavetables},
};
 
 use std::{
    fs,
    sync::Arc,
};

 const SAMPLE_RATE: u16 = 44_100; // Audio sample rate in Hz
 
 fn main() {
     // Create a collection of wavetables and load them from files
    let mut wt_on_heap = BoxedWavetables::new();
    for id in 0..10 {
        let filename = format!("examples/soundbank/soundbank_pure_elektro/src/wav{}.raw", id);
        let contents = fs::read(filename).unwrap();
        let bytes: &[u8] = &contents;
        wt_on_heap.add(BoxedWavetable::new(bytes));
    }
    // Wrap wavetables in an Arc for thread-safe sharing
    let wt = Arc::new(wt_on_heap);
    
    // Load a synth patch from a JSON file
    let patch = serde_json::from_slice(include_bytes!("patches/bass.json")).unwrap();

    // Create a new synthesizer instance with specified parameters
    let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE as u16, &patch, Arc::clone(&wt));

    // Trigger a note
    synth.note_on(60, 100);

    loop {
        let _sample:[i16;2] = synth.clock_and_output();
        // do something with the sample, stream it to a audio device for example
        break;
    }
    
 }
```

Run Little Weirdo as a MIDI device while outputting to a local Audio Output, check out [Little Weirdo Streaming Audio With MIDI](examples/little_weirdo_streaming_audio_with_midi.rs)

If you just want to play with a patch for Little Weirdo, check out [Little Weirdo Patch Tester](examples/little_weirdo_patch_tester.rs)

To generate you own Soundbank, download some Wavetable samples from AKWF and run [Little Weirdo Generate Soundbank](examples/little_weirdo_generate_soundbanks.rs)

## Performance

The performance tests on real embedded hardware can be found here [Little Weirdo ESP32](https://github.com/hi-squeaky-things/little-weirdo-esp32)

## Patches

Patches can be fully programmed in Rust or loaded using JSON (Testing) or Postcard (Embedded Devices).  Checkout the [patches](examples/patches/)

🎹 Listen to the Ebass patch (unmute the audio 🔇 ➡️ 🔊) :

https://github.com/user-attachments/assets/56b9666d-ab76-4716-8fe9-58ffc7642058

🎹 SuperSaw with 4 oscillators enabled :

https://github.com/user-attachments/assets/149d2a91-86f8-4a7f-bbc5-ff8ae06aa395

## Credits

- [Small Braille ASCII Font](https://patorjk.com/software/taag/#p=display&f=Small+Braille&t=LITTLE+WEIRDO&x=rainbow1&v=1&h=1&w=80&we=false)
- [Wavetable samples by Adventure Kid Waveforms ](https://github.com/KristofferKarlAxelEkstrand/AKWF-FREE)


