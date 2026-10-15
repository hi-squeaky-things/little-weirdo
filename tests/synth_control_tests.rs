use little_weirdo::synth::{
    control::SynthControl,
    data::{
        patches::{BoxedPatch, BoxedPatches},
        waveforms::{BoxedWaveform, BoxedWaveforms},
    },
    patch::Patch,
    Synth,
};
use std::sync::Arc;

#[test]
fn live_controls_are_available_through_a_separate_handle() {
    let mut patch: Patch = serde_json::from_slice(include_bytes!(
        "../examples/soundbank/synth/patches/original/bass.json"
    ))
    .unwrap();
    patch.fm_config.enabled = true;

    let mut patches = BoxedPatches::new();
    patches.add(BoxedPatch::new(patch));

    let mut waveforms = BoxedWaveforms::new();
    waveforms.add(BoxedWaveform::new(include_bytes!(
        "../examples/soundbank/synth/waveforms/src/000_sample.raw"
    )));
    waveforms.add(BoxedWaveform::new(include_bytes!(
        "../examples/soundbank/synth/waveforms/src/001_sample.raw"
    )));
    waveforms.add(BoxedWaveform::new(include_bytes!(
        "../examples/soundbank/synth/waveforms/src/002_sample.raw"
    )));
    waveforms.add(BoxedWaveform::new(include_bytes!(
        "../examples/soundbank/synth/waveforms/src/003_sample.raw"
    )));
    waveforms.add(BoxedWaveform::new(include_bytes!(
        "../examples/soundbank/synth/waveforms/src/004_sample.raw"
    )));

    let mut synth = Synth::new(44_100, 0, Arc::new(patches), Arc::new(waveforms));
    {
        let mut control: SynthControl<'_> = synth.control();
        assert_eq!(control.fm_carrier_waveform(), 0);
        control.adjust_fm_carrier_waveform(true);
        assert_eq!(control.fm_carrier_waveform(), 1);
    }

    synth.note_on(60, 127);
    let _ = synth.clock_and_output();
}
