use std::{fs::File, io::Write};

use little_weirdo::{
    effects::{
        bitcrunch::BitcrunchConfiguration,
        delay::DelayConfiguration,
        overdrive::{KindOfOverdrive, OverdriveConfiguration},
    },
    sampler::patch::Patch,
};

fn main() {
    let patch = Patch {
        name: "test".to_string(),
        overdrive_config: OverdriveConfiguration {
            threshold: 1000,
            kind: KindOfOverdrive::Hard,
            enabled: true,
        },
        bitcrunch_config: BitcrunchConfiguration { enabled: true },
        delay_config: DelayConfiguration {
            enabled: true,
            delay_time: 4000,
            feedback: true,
            mix_percentage: 70,
            feedback_percentage: 70,
        },
        drums: true,
        sample_map: 0,
        loop_start: 0,
        loop_end: 0,
        one_shot: true,
        base_key: 74,
        env_config: little_weirdo::synth::envelope::EnvelopConfiguration {
            attack_time: 0,
            decay_time: 0,
            release_time: 0,
            sustain_level: 0,
        },
        zones: vec![little_weirdo::sampler::patch::Zone {
            start_note: 0,
            end_note: 127,
            sample_map: 0,
            base_key: 60,
            loop_start: 0,
            loop_end: 0,
            one_shot: true,
        }],
        velocity_layers: false,
        num_velocity_layers: 0,
        round_robin: false,
        round_robin_count: 0,
    };
    let output = serde_json::to_string(&patch);

    let mut json_patch_file = File::create("output.json").expect("Failed to create output file");
    json_patch_file
        .write_all(&output.unwrap().as_bytes())
        .expect("Failed to write to file");
}
