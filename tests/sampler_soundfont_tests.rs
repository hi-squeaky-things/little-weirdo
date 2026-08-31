

use little_weirdo::sampler::{
    self, audio_sampler::{BoxedSample, BoxedSamples}, data::{patches::{BoxedSamplerPatch, BoxedSamplerPatches, Patches}}, patch::Patch,
};
use little_weirdo::{
    effects::{bitcrunch::BitcrunchConfiguration, delay::DelayConfiguration, overdrive::{OverdriveConfiguration, KindOfOverdrive}},
    synth::envelope::EnvelopConfiguration,
};

#[test]
fn test_sampler_with_zones() {
    // Create a simple patch with zones
    let patch = Patch {
        name: "Test Patch with Zones".to_string(),
        drums: false,
        sample_map: 0,
        loop_start: 0,
        loop_end: 0,
        one_shot: true,
        base_key: 60,
        overdrive_config: OverdriveConfiguration {
            threshold: 30000,
            kind: KindOfOverdrive::Hard,
            enabled: false,
        },
        bitcrunch_config: BitcrunchConfiguration {
            enabled: false,
        },
        delay_config: DelayConfiguration {
            enabled: false,
            delay_time: 500,
            mix_percentage: 50,
            feedback: false,
            feedback_percentage: 50,
        },
        env_config: EnvelopConfiguration {
            attack_time: 0,
            decay_time: 1000,
            sustain_level: 10000,
            release_time: 1000,
        },
        zones: vec![
            sampler::patch::Zone {
                start_note: 21,
                end_note: 48,
                sample_map: 0,
                base_key: 45,
                loop_start: 0,
                loop_end: 0,
                one_shot: true,
            },
            sampler::patch::Zone {
                start_note: 49,
                end_note: 72,
                sample_map: 1,
                base_key: 60,
                loop_start: 0,
                loop_end: 0,
                one_shot: true,
            },
        ]
    };

    // Create patches collection
    let mut patches = BoxedSamplerPatches::new();
    patches.add(BoxedSamplerPatch::new(patch));

    // Create samples collection (empty for this test)
    let mut samples = BoxedSamples::new();
    samples.add(BoxedSample::new(vec![]));
    samples.add(BoxedSample::new(vec![]));

    // Create sampler
    let _sampler = sampler::Sampler::new(
        44100,
        0,
        std::sync::Arc::new(patches),
        std::sync::Arc::new(samples),
    );

    // Test that sampler was created successfully
    // Note: sample_map and drums are private fields, so we can't test them directly
    // but the sampler creation should succeed without panicking
}


#[test]
fn test_zone_selection() {
    // Create a patch with multiple zones
    let patch = Patch {
        name: "Test Zone Selection".to_string(),
        drums: false,
        sample_map: 0,
        loop_start: 0,
        loop_end: 0,
        one_shot: true,
        base_key: 60,
        overdrive_config: OverdriveConfiguration {
            threshold: 30000,
            kind: KindOfOverdrive::Hard,
            enabled: false,
        },
        bitcrunch_config: BitcrunchConfiguration {
            enabled: false,
        },
        delay_config: DelayConfiguration {
            enabled: false,
            delay_time: 500,
            mix_percentage: 50,
            feedback: false,
            feedback_percentage: 50,
        },
        env_config: EnvelopConfiguration {
            attack_time: 0,
            decay_time: 1000,
            sustain_level: 10000,
            release_time: 1000,
        },
        zones: vec![
            sampler::patch::Zone {
                start_note: 21,
                end_note: 48,
                sample_map: 0,
                base_key: 45,
                loop_start: 0,
                loop_end: 0,
                one_shot: true,
            },
            sampler::patch::Zone {
                start_note: 49,
                end_note: 72,
                sample_map: 1,
                base_key: 60,
                loop_start: 0,
                loop_end: 0,
                one_shot: true,
            },
            sampler::patch::Zone {
                start_note: 73,
                end_note: 108,
                sample_map: 2,
                base_key: 75,
                loop_start: 0,
                loop_end: 0,
                one_shot: true,
            },
        ]
    };

    // Create patches collection
    let mut patches = BoxedSamplerPatches::new();
    patches.add(BoxedSamplerPatch::new(patch));

    // Create samples collection (empty for this test)
    let mut samples = BoxedSamples::new();
    samples.add(BoxedSample::new(vec![]));
    samples.add(BoxedSample::new(vec![]));
    samples.add(BoxedSample::new(vec![]));

    // Create sampler
    let sampler = sampler::Sampler::new(
        44100,
        0,
        std::sync::Arc::new(patches),
        std::sync::Arc::new(samples),
    );

    // Test zone selection logic
    let patch_ref = sampler.patches.get_patches_reference(0);
    
    // Test note in first zone
    let (sample_id, base_key, _, _, _) = sampler.get_zone_params(36, patch_ref);
    assert_eq!(sample_id, 0);
    assert_eq!(base_key, 45);
    
    // Test note in second zone
    let (sample_id, base_key, _, _, _) = sampler.get_zone_params(60, patch_ref);
    assert_eq!(sample_id, 1);
    assert_eq!(base_key, 60);
    
    // Test note in third zone
    let (sample_id, base_key, _, _, _) = sampler.get_zone_params(84, patch_ref);
    assert_eq!(sample_id, 2);
    assert_eq!(base_key, 75);
}