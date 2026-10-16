use little_weirdo::synth::{
    data::{
        patches::{BoxedPatch, BoxedPatches},
        waveforms::{BoxedWaveform, BoxedWaveforms},
    },
    noise::NoiseKind,
    patch::{Patch, SynthMode},
    Synth, AMOUNT_OF_VOICES,
};
use std::sync::Arc;

#[test]
fn test_patch_deserialization_json() {
    // Test loading a patch from JSON file.
    let patch: Patch = serde_json::from_slice(include_bytes!(
        "../examples/soundbank/synth/patches/original/bass.json"
    ))
    .unwrap();

    // Verify the patch was loaded successfully.
    assert!(!patch.voices.is_empty());
    assert!(!patch.envelops.is_empty());
    assert!(!patch.lfos.is_empty());
    assert_eq!(patch.voices[0].noise, NoiseKind::None);
    assert!(patch.flanger_config.enabled);
    assert_eq!(patch.flanger_config.base_delay_time_ms, 10);
    assert_eq!(patch.flanger_config.depth_ms, 5);
    assert_eq!(patch.flanger_config.rate_hz, 5);
    assert_eq!(patch.flanger_config.mix_percentage, 40);
    assert!(!patch.fm_config.enabled);
    assert_eq!(patch.fm_config.carrier_waveform, 0);
    assert_eq!(patch.fm_config.modulator_waveform, 0);
    assert_eq!(patch.fm_config.modulator_ratio, 1);
    assert_eq!(patch.fm_config.modulation_depth_hz, 100);
}

#[test]
fn test_fm_patch_postcard_roundtrip() {
    let mut patch: Patch = serde_json::from_slice(include_bytes!(
        "../examples/soundbank/synth/patches/original/bass.json"
    ))
    .unwrap();
    patch.fm_config.enabled = true;
    patch.fm_config.carrier_waveform = 3;
    patch.fm_config.modulator_ratio = 2;
    patch.fm_config.modulation_depth_hz = 350;

    let mut encoded = [0; 512];
    let encoded = postcard::to_slice(&patch, &mut encoded).unwrap();
    let decoded: Patch = postcard::from_bytes(encoded).unwrap();

    assert!(decoded.fm_config.enabled);
    assert_eq!(decoded.fm_config.carrier_waveform, 3);
    assert_eq!(decoded.fm_config.modulator_ratio, 2);
    assert_eq!(decoded.fm_config.modulation_depth_hz, 350);
}

#[test]
fn fm_example_patch_is_available_as_json_and_postcard() {
    let json_patch: Patch = serde_json::from_slice(include_bytes!(
        "../examples/soundbank/synth/patches/original/fm_basic.json"
    ))
    .unwrap();
    let postcard_patch: Patch = postcard::from_bytes(include_bytes!(
        "../examples/soundbank/synth/patches/src/fm_basic.lwp"
    ))
    .unwrap();

    for patch in [&json_patch, &postcard_patch] {
        assert_eq!(patch.name, "fm_basic");
        assert!(patch.fm_config.enabled);
        assert_eq!(patch.fm_config.carrier_waveform, 0);
        assert_eq!(patch.fm_config.modulator_waveform, 0);
        assert_eq!(patch.fm_config.modulator_ratio, 3);
        assert_eq!(patch.fm_config.modulation_depth_hz, 200);
        assert_eq!(patch.synth_config.mode, SynthMode::OctoPoly);
    }
}

#[test]
fn fm_carrier_waveform_is_serialized_in_postcard_asset() {
    let patch: Patch = postcard::from_bytes(include_bytes!(
        "../examples/soundbank/synth/patches/src/fm_basic.lwp"
    ))
    .unwrap();

    assert_eq!(patch.fm_config.carrier_waveform, 0);
    assert_eq!(patch.fm_config.modulator_waveform, 0);
    assert_eq!(patch.fm_config.modulator_ratio, 3);
    assert_eq!(patch.fm_config.modulation_depth_hz, 200);
}

#[test]
fn test_patch_deserialization_json_accepts_fm_configuration() {
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../examples/soundbank/synth/patches/original/bass.json"
    ))
    .unwrap();
    value["fm_config"] = serde_json::json!({
        "enabled": true,
        "modulator_waveform": 0,
        "modulator_ratio": 2,
        "modulation_depth_hz": 350
    });

    let patch: Patch = serde_json::from_value(value).unwrap();
    assert!(patch.fm_config.enabled);
    assert_eq!(patch.fm_config.carrier_waveform, 0);
    assert_eq!(patch.fm_config.modulator_waveform, 0);
    assert_eq!(patch.fm_config.modulator_ratio, 2);
    assert_eq!(patch.fm_config.modulation_depth_hz, 350);
}

#[test]
fn fm_patch_modulates_the_carrier_in_the_synth() {
    let mut fm_patch: Patch = serde_json::from_slice(include_bytes!(
        "../examples/soundbank/synth/patches/original/bass.json"
    ))
    .unwrap();
    fm_patch.fm_config.enabled = true;
    fm_patch.fm_config.modulator_waveform = 0;
    fm_patch.fm_config.modulator_ratio = 2;
    fm_patch.fm_config.modulation_depth_hz = 500;

    let mut dry_patch = fm_patch.clone();
    dry_patch.fm_config.enabled = false;

    let waveforms = {
        let mut waveforms = BoxedWaveforms::new();
        for bytes in [
            include_bytes!("../examples/soundbank/synth/waveforms/src/000_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/001_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/002_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/003_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/004_sample.raw").as_slice(),
        ] {
            waveforms.add(BoxedWaveform::new(bytes));
        }
        Arc::new(waveforms)
    };

    let mut fm_patches = BoxedPatches::new();
    fm_patches.add(BoxedPatch::new(fm_patch));
    let mut fm_synth = Synth::new(44_100, 0, Arc::new(fm_patches), Arc::clone(&waveforms));

    let mut dry_patches = BoxedPatches::new();
    dry_patches.add(BoxedPatch::new(dry_patch));
    let mut dry_synth = Synth::new(44_100, 0, Arc::new(dry_patches), waveforms);

    fm_synth.note_on(60, 127);
    dry_synth.note_on(60, 127);
    let differs = (0..1_000).any(|_| fm_synth.clock_and_output() != dry_synth.clock_and_output());

    assert!(differs);
}

#[test]
fn fm_carrier_waveform_is_used_and_can_be_changed_at_runtime() {
    let mut patch: Patch = serde_json::from_slice(include_bytes!(
        "../examples/soundbank/synth/patches/original/bass.json"
    ))
    .unwrap();
    patch.fm_config.enabled = true;
    patch.fm_config.carrier_waveform = 0;
    patch.fm_config.modulator_waveform = 0;
    patch.fm_config.modulator_ratio = 2;
    patch.fm_config.modulation_depth_hz = 0;
    patch.mixer_config.gain_voices = [100, 0, 0, 0, 0, 0, 0, 0];
    patch.mixer_config.gain_main = 100;
    patch.routering_config.voices_to_envelop = [0; AMOUNT_OF_VOICES];
    patch.filter_config.enabled = false;
    patch.overdrive_config.enabled = false;
    patch.bitcrunch_config.enabled = false;
    patch.delay_config.enabled = false;
    patch.flanger_config.enabled = false;

    let mut alternate_carrier_patch = patch.clone();
    alternate_carrier_patch.fm_config.carrier_waveform = 1;
    let waveforms = {
        let mut waveforms = BoxedWaveforms::new();
        for bytes in [
            include_bytes!("../examples/soundbank/synth/waveforms/src/000_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/001_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/002_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/003_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/004_sample.raw").as_slice(),
        ] {
            waveforms.add(BoxedWaveform::new(bytes));
        }
        Arc::new(waveforms)
    };
    let mut patches = BoxedPatches::new();
    patches.add(BoxedPatch::new(patch));
    let mut synth = Synth::new(44_100, 0, Arc::new(patches), Arc::clone(&waveforms));
    let mut alternate_patches = BoxedPatches::new();
    alternate_patches.add(BoxedPatch::new(alternate_carrier_patch));
    let mut alternate_synth = Synth::new(44_100, 0, Arc::new(alternate_patches), waveforms);

    assert_eq!(synth.control().fm_carrier_waveform(), 0);
    synth.note_on(60, 127);
    alternate_synth.note_on(60, 127);
    let output_differs =
        (0..1_000).any(|_| synth.clock_and_output() != alternate_synth.clock_and_output());

    assert!(output_differs);

    {
        let mut control = synth.control();
        control.adjust_fm_carrier_waveform(true);
        assert_eq!(control.fm_carrier_waveform(), 1);
        control.adjust_fm_carrier_waveform(false);
        assert_eq!(control.fm_carrier_waveform(), 0);
    }
}

#[test]
fn fm_max_velocity_does_not_mix_unallocated_carrier_slots() {
    let mut patch: Patch = serde_json::from_slice(include_bytes!(
        "../examples/soundbank/synth/patches/original/bass.json"
    ))
    .unwrap();
    patch.fm_config.enabled = true;
    patch.fm_config.modulator_waveform = 0;
    patch.fm_config.modulator_ratio = 2;
    patch.fm_config.modulation_depth_hz = 500;
    patch.mixer_config.gain_voices = [100, 0, 100, 0, 100, 0, 100, 0];
    patch.mixer_config.gain_main = 100;
    patch.routering_config.voices_to_envelop = [0; AMOUNT_OF_VOICES];
    patch.filter_config.enabled = false;
    patch.overdrive_config.enabled = false;
    patch.bitcrunch_config.enabled = false;
    patch.delay_config.enabled = false;
    patch.flanger_config.enabled = false;

    let mut single_carrier_patch = patch.clone();
    single_carrier_patch.mixer_config.gain_voices = [100, 0, 0, 0, 0, 0, 0, 0];

    let waveforms = {
        let mut waveforms = BoxedWaveforms::new();
        for bytes in [
            include_bytes!("../examples/soundbank/synth/waveforms/src/000_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/001_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/002_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/003_sample.raw").as_slice(),
            include_bytes!("../examples/soundbank/synth/waveforms/src/004_sample.raw").as_slice(),
        ] {
            waveforms.add(BoxedWaveform::new(bytes));
        }
        Arc::new(waveforms)
    };
    let mut patches = BoxedPatches::new();
    patches.add(BoxedPatch::new(patch));
    let mut synth = Synth::new(44_100, 0, Arc::new(patches), Arc::clone(&waveforms));

    let mut single_carrier_patches = BoxedPatches::new();
    single_carrier_patches.add(BoxedPatch::new(single_carrier_patch));
    let mut single_carrier_synth =
        Synth::new(44_100, 0, Arc::new(single_carrier_patches), waveforms);

    synth.note_on(60, 127);
    single_carrier_synth.note_on(60, 127);
    for _ in 0..4_000 {
        assert_eq!(
            synth.clock_and_output(),
            single_carrier_synth.clock_and_output()
        );
    }
}

#[test]
fn test_patch_deserialization_json_accepts_flanger_configuration() {
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../examples/soundbank/synth/patches/original/bass.json"
    ))
    .unwrap();
    value["flanger_config"] = serde_json::json!({
        "enabled": true,
        "base_delay_time_ms": 2,
        "depth_ms": 1,
        "rate_hz": 3,
        "mix_percentage": 40
    });

    let patch: Patch = serde_json::from_value(value).unwrap();
    assert!(patch.flanger_config.enabled);
    assert_eq!(patch.flanger_config.base_delay_time_ms, 2);
    assert_eq!(patch.flanger_config.depth_ms, 1);
    assert_eq!(patch.flanger_config.rate_hz, 3);
    assert_eq!(patch.flanger_config.mix_percentage, 40);
}

#[test]
fn test_synth_mode_polyphony_matches_expected_note_limits() {
    assert_eq!(SynthMode::Mono as usize, 1);
    assert_eq!(SynthMode::BiPoly as usize, 2);
    assert_eq!(SynthMode::QuadPoly as usize, 4);
    assert_eq!(SynthMode::OctoPoly as usize, 8);

    assert_eq!(SynthMode::Mono.voices_per_note(), 8);
    assert_eq!(SynthMode::BiPoly.voices_per_note(), 4);
    assert_eq!(SynthMode::QuadPoly.voices_per_note(), 2);
    assert_eq!(SynthMode::OctoPoly.voices_per_note(), 1);

    assert_eq!(SynthMode::Mono.max_active_notes(), 1);
    assert_eq!(SynthMode::BiPoly.max_active_notes(), 2);
    assert_eq!(SynthMode::QuadPoly.max_active_notes(), 4);
    assert_eq!(SynthMode::OctoPoly.max_active_notes(), 8);
}
