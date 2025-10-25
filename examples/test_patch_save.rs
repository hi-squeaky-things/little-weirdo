use std::{fs::File, io::{Read, Write}, path::Path};
use little_weirdo::synth::{effects::{bitcrunch::BitcrunchConfiguration, filter::{FilterConfig, KindOfFilter}, overdrive::{KindOfOverdrive, OverdriveConfiguration}}, envelope::EnvelopConfiguration, mixer::MixerConfiguration, patch::{Patch, SynthConfiguration, SynthMode}, router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute}, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig}};
use serde_json;


fn main()  {

    let patch = Patch {
        synth_config: SynthConfiguration {
            mode: SynthMode::Mono,
        },
        voices: [
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 3,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 6,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
        ],
        envelops: [
            EnvelopConfiguration {
                attack_time: 5,
                decay_time: 10,
                release_time: 20,
                sustain_level: 80,
            },
            EnvelopConfiguration {
                attack_time: 200,
                decay_time: 100,
                release_time: 100,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
        ],
        lfos: [
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 0,
                time: 10,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 1,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 4,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 4,
                time: 200,
            },
        ],
        routering_config: RoutingConfiguration {
            voices_to_envelop: [
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 3 },
                VoiceToEnvelopRoute { env: 4 },
                VoiceToEnvelopRoute { env: 5 },
                VoiceToEnvelopRoute { env: 6 },
                VoiceToEnvelopRoute { env: 7 },
            ],
            voice_to_lfo: [
                VoiceToLFORoute {
                    enable: false,
                    voices: [0, 1],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
            ],
            lfo_to_filter: false,
            lfo_to_freq: true,
            lfo_to_freq_amount: 5, 
        },
        filter_config: FilterConfig {
            cutoff_frequency: 2000,
            resonance: 0,
            enabled: true,
            kind_of_filter: KindOfFilter::Low,
        },
        mixer_config: MixerConfiguration {
            gain_voices: [50, 10, 0, 0, 0, 0, 0, 0],
            gain_main: 100,
        },
        overdrive_config: OverdriveConfiguration {
            threshold: 2000,
            kind: KindOfOverdrive::Softer,
            enabled: false,
        },
        bitcrunch_config: BitcrunchConfiguration {
            enabled: false,
        }
    };


    println!("{:?}",patch.synth_config );
   //  let json = serde_json::to_string(&patch)?;
     let file = File::create("patch.json").expect("Failed to open file");;
    serde_json::to_writer(file, &patch);

    let mut buf = [0u8; 163];
    
    match postcard::to_slice(&patch, &mut buf) {
        Ok(_result) => {println!("Serialize Ok")},
        Err(error) => {println!("{:?}", error)},
    }

    let file_name = Path::new("./").join("patch.p");
    let mut file =  File::create(file_name).expect("Failed to open file");
   let _ = file.write(&buf);


    let mut buf2 = [0u8; 163];
   let _ = file.read(&mut buf2);

   let test:Patch = postcard::from_bytes(&buf2).unwrap();
   println!("{:?}",test.synth_config );
}