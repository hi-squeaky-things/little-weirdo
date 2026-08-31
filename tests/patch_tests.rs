use little_weirdo::synth::patch::{Patch, SynthMode};
use postcard;

#[test]
fn test_patch_deserialization_json() {
    // Test loading a patch from JSON file.
    let patch: Patch =
        serde_json::from_slice(include_bytes!("../examples/soundbank/synth/patches/original/bass.json"))
            .unwrap();

    // Verify the patch was loaded successfully.
    assert!(!patch.voices.is_empty());
    assert!(!patch.envelops.is_empty());
    assert!(!patch.lfos.is_empty());
}

#[test]
fn test_patch_deserialization_postcard() {
    // Test loading a patch from a postcard file.
    let patch: Patch =
        postcard::from_bytes(include_bytes!("../examples/soundbank/synth/patches/src/bass.lwp")).unwrap();

    // Verify the patch was loaded successfully.
    assert!(!patch.voices.is_empty());
    assert!(!patch.envelops.is_empty());
    assert!(!patch.lfos.is_empty());
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
