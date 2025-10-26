use little_weirdo::synth::patch::Patch;
use postcard;

#[test]
fn test_patch_deserialization_json() {
    // Test loading a patch from JSON file
    let patch: Patch = serde_json::from_slice(include_bytes!("../examples/patches/bass.json")).unwrap();
    
    // Verify the patch was loaded successfully
    assert!(!patch.voices.is_empty());
    assert!(!patch.envelops.is_empty());
    assert!(!patch.lfos.is_empty());
}

#[test]
fn test_patch_deserialization_postcard() {
    // Test loading a patch from JSON file
    let patch: Patch = postcard::from_bytes(include_bytes!("../examples/patches/bass.lwp")).unwrap();
    
    // Verify the patch was loaded successfully
    assert!(!patch.voices.is_empty());
    assert!(!patch.envelops.is_empty());
    assert!(!patch.lfos.is_empty());
}