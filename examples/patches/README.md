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

# Synthesizer Patch Configuration

Patches define the complete configuration of the synthesizer, including voice settings, envelopes, LFOs, filters, mixer, effects, and routing. The patch structure is defined by the `lw_patch.json` schema.

## Voice Configuration

Each patch contains 8 voices, each with:
- `soundbank_index`: Selects wavetable (0-9) or noise (255)
- `glide`: Enables pitch glide
- `glide_rate`: Controls glide speed
- `detune`: Semitone detuning
- `freq_detune`: Frequency detuning in Hz

## Envelope Configuration

Eight ADSR envelopes control sound shaping:
- `attack_time`: Attack time in samples
- `decay_time`: Decay time in samples
- `release_time`: Release time in samples
- `sustain_level`: Sustain level percentage (0-100)

## Low Frequency Oscillators (LFOs)

Four LFOs provide modulation:
- `soundbank_index`: Wavetable selection (0-9)
- `time`: Modulation rate/frequency

## Filter Configuration

Global filter settings:
- `cutoff_frequency`: Filter cutoff (0-65535)
- `resonance`: Filter resonance (0-65535)
- `kind_of_filter`: Filter type ('low', 'high', 'band', 'notch')
- `enabled`: Enable/disable filter

## Mixer Configuration

Controls output levels:
- `gain_voices`: Per-voice gain (0-100)
- `gain_main`: Main output gain (0-100)

## Effects Configuration

- **Overdrive**: Distortion with threshold, type ('Soft', 'Hard', 'Softer'), and enable flag
- **Bitcrunch**: 8-bit style bit reduction effect

## Routing Configuration

Manages signal flow:
- `voices_to_envelop`: Voice-to-envelope mapping
- `lfo_to_voice`: LFO-to-voice routing with enable flags
- `lfo_to_filter`: LFO #0 modulates filter cutoff
- `lfo_to_freq`: LFO #0 modulates oscillator frequencies
- `lfo_to_freq_amount`: Amount of frequency modulation in Hz

## Synthesizer Mode

Sets polyphony mode:
- `mode`: 'Mono', 'BiPoly', 'QuadPoly', or 'OctoPoly'

### Mode Details

- **Mono** (`mode: 1`): Only one voice is active at a time. You can use up to 8 oscillators per voice. Creating for instance an insane SuperSaw.
- **BiPoly** (`mode: 2`): Two voices can be active at any time. You can use up to 4 oscillators per voice.
- **QuadPoly** (`mode: 3`): Four voices can be active at any time. You can use up to 2 oscillators per voice.
- **OctoPoly** (`mode: 4`): Eight voices can be active at any time. You can use only 1 oscillator per voice.

The mode affects how many voices are allocated and how the voices are configured. The voices are distributed across the available voice slots in a round-robin fashion.