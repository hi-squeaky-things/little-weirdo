
use cpal::Sample;
use little_weirdo::synth::{self, data::wavetables::{BoxedWavetable, BoxedWavetables}, effects::{filter::{FilterConfig, KindOfFilter}, overdrive::{KindOfOverdrive, OverdriveConfiguration}}, envelope::EnvelopConfiguration, mixer::MixerConfiguration, patch::{Patch, SynthConfiguration, SynthMode}, router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute}, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig}};


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
   
    let mut synth: synth::Synth =
    synth::Synth::new(SAMPLE_RATE as u16, &SOUND_BANK_PURE_ELEKTRO.patches[1],&SOUND_BANK_PURE_ELEKTRO);
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