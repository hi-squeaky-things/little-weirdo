
use cpal::Sample;
use little_weirdo::synth::{self, math::{self, percentage}, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillator, WaveTableOscillatorConfig}, Clockable};
use little_weirdo_soundbanks::{soundbanks::SOUND_BANK_PURE_ELEKTRO};
use little_weirdo_soundbanks::patches::Patches;

const SAMPLE_RATE:u16 = 44_100;
const CLIPPING:u16 = 32_000;
const HEADROOM:f64 = -12.0;

fn main() {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("lfo_test.wav", spec).unwrap();

    let mut lfo_config = WaveTableLoFreqOscillatorConfig {
        soundbank_index: 4,
        time: 5, // 0.5 sec 
    };

    let mut osc_conf = WaveTableOscillatorConfig {
        soundbank_index: 4,
        glide: false,
        glide_rate: 0,
        detune: 0,
        freq: 220,
    };

    let mut lfo: WaveTableOscillator = WaveTableOscillator::new_lfo(lfo_config,&SOUND_BANK_PURE_ELEKTRO ,SAMPLE_RATE);
    let mut osc:WaveTableOscillator = WaveTableOscillator::new(osc_conf, &SOUND_BANK_PURE_ELEKTRO, SAMPLE_RATE);
        for n in 0..SAMPLE_RATE {
            let output_lfo = lfo.clock(None);
            writer.write_sample(output_lfo).unwrap();
        
            let output_lfo_lifted:i32 = output_lfo as i32 + i16::MAX as i32;
            let multiplier = (((output_lfo_lifted as u32) * 100) / u16::MAX as u32);
            let output_osc = osc.clock(None);
            
            writer.write_sample(percentage(output_osc,multiplier as i16)).unwrap();
        }
    }


