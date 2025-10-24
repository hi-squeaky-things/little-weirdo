
use cpal::Sample;
use little_weirdo::synth::{self, data::wavetables::{BoxedWavetable, BoxedWavetables}, effects::{bitcrunch::BitcrunchConfiguration, filter::{FilterConfig, KindOfFilter}, overdrive::{KindOfOverdrive, OverdriveConfiguration}}, envelope::EnvelopConfiguration, math::percentage, mixer::MixerConfiguration, patch::{Patch, SynthConfiguration, SynthMode}, router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute}, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillator, WaveTableOscillatorConfig}, Clockable};
use std::{fs, mem, rc::Rc, sync::{mpsc, Arc}};
use little_weirdo::synth::{Synth};

const SAMPLE_RATE:u16 = 44_100;
const CLIPPING:u16 = 32_000;
const HEADROOM:f64 = -12.0;

fn main() {

        let mut wt_on_heap = BoxedWavetables::new();
    for id in 0..10 {
        let filename = format!("examples/wav{}.lwt", id);
        let contents = fs::read(filename).unwrap();
        let bytes: &[u8] = &contents;
        wt_on_heap.add(BoxedWavetable::new(bytes));
   
    }
    let wt = Arc::new(wt_on_heap);


   


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
        freq_detune: 0,
    };

    let mut lfo: WaveTableOscillator = WaveTableOscillator::new_lfo(lfo_config,SAMPLE_RATE, Arc::clone(&wt) );
    let mut osc:WaveTableOscillator = WaveTableOscillator::new(osc_conf,SAMPLE_RATE, Arc::clone(&wt));
        for n in 0..SAMPLE_RATE {
            let output_lfo = lfo.clock(None);
            writer.write_sample(output_lfo).unwrap();
        
            let output_lfo_lifted:i32 = output_lfo as i32 + i16::MAX as i32;
            let multiplier = (((output_lfo_lifted as u32) * 100) / u16::MAX as u32);
            let output_osc = osc.clock(None);
            
            writer.write_sample(percentage(output_osc,multiplier as i16)).unwrap();
        }
    }


