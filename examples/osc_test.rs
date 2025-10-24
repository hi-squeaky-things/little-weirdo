
use std::{fs, rc::Rc, sync::Arc};

use cpal::Sample;
use little_weirdo::synth::{self, data::wavetables::{BoxedWavetable, BoxedWavetables}, effects::{bitcrunch::BitcrunchConfiguration, filter::{FilterConfig, KindOfFilter}, overdrive::{KindOfOverdrive, OverdriveConfiguration}}, envelope::EnvelopConfiguration, mixer::MixerConfiguration, patch::{Patch, SynthConfiguration, SynthMode}, router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute}, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig}};


const SAMPLE_RATE:u16 = 48000;
const CLIPPING:u16 = 32_000;
const HEADROOM:f64 = -12.0;

fn main() {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("osc_test.wav", spec).unwrap();
   
    let mut wt_on_heap = BoxedWavetables::new();
    for id in 0..10 {
        let filename = format!("examples/wav{}.lwt", id);
        let contents = fs::read(filename).unwrap();
        let bytes: &[u8] = &contents;
        wt_on_heap.add(BoxedWavetable::new(bytes));
   
    }
    let wt = Arc::new(wt_on_heap);

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


   
    let mut synth: synth::Synth = synth::Synth::new(
        SAMPLE_RATE as u16,
        &patch,
        Arc::clone(&wt),
    );
    
 
    let mut clipped: [i32;2] = [0;2];
    let note = 46;
   // for note in 60 {
        synth.note_on(note, 100);
        let mut total: f64 = 0.0;
        for n in 0..SAMPLE_RATE {
            let output = synth.clock_and_output(); 
            writer.write_sample(output[0]).unwrap();
            let left:f64 = Sample::from_sample(output[0]);
            total += left * left;


            if output[0].abs() > CLIPPING as i16 { clipped[0] = clipped[0] + 1;  }
            if output[1].abs() > CLIPPING as i16 { clipped[0] = clipped[0] + 1;  }
        }
        synth.note_off(note);
        for n in 0..SAMPLE_RATE {
            let output = synth.clock_and_output(); 
            writer.write_sample(output[0]).unwrap();
        }
        let rms = total / SAMPLE_RATE as f64;
        let db = decibel(rms).round();
        print!("[{:?}]::", note);
        print!("{:?}::", db);
        if (db > HEADROOM) {
            print!("H") 
        }

        if (clipped[0] > 0) {
            print!("C");
       
        } else {
            print!(".");
        }
        
        print!("-");
        if (note % 8 == 0) {
            println!("");
    }
    println!("");

}

fn decibel(rms: f64) -> f64 {
    // Convert RMS value to decibels using a simple formula\
    10.0 * rms.log10()
}