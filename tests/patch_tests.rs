use little_weirdo::synth::{
    noise::NoiseKind,
    patch::{Patch, SynthMode},
};
use postcard;

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
    assert_eq!(patch.flanger_config.rate_hz, 1);
    assert_eq!(patch.flanger_config.mix_percentage, 30);
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
