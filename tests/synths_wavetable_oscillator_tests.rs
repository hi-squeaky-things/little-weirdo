use std::{fs, sync::Arc};

use little_weirdo::synth::noise::NoiseKind;
use little_weirdo::synth::{
    data::waveforms::{BoxedWaveform, BoxedWaveforms},
    waveform_oscillator::{WaveformOscillator, WaveformOscillatorConfig},
};

fn load_sine_wavetable() -> Arc<BoxedWaveforms> {
    let raw = fs::read("examples/soundbank/synth/waveforms/src/000_sample.raw").unwrap();
    let mut waveforms = BoxedWaveforms::new();
    waveforms.add(BoxedWaveform::new(&raw));
    Arc::new(waveforms)
}

#[test]
fn wavetable_oscillator_outputs_samples_from_the_selected_table() {
    let wavetables = load_sine_wavetable();
    let config = WaveformOscillatorConfig {
        soundbank_index: 0,
        glide: false,
        glide_rate: 0,
        detune: 0,
        freq_detune: 0,
        grains: false,
        grains_seq: [0; 8],
        noise: NoiseKind::None,
    };

    let _osc = WaveformOscillator::new(config, 1000, Arc::clone(&wavetables));
}

#[test]
fn wavetable_oscillator_can_change_frequency() {
    let wavetables = load_sine_wavetable();
    let config = WaveformOscillatorConfig {
        soundbank_index: 0,
        glide: false,
        glide_rate: 0,
        detune: 0,
        freq_detune: 0,
        grains: false,
        grains_seq: [0; 8],
        noise: NoiseKind::None,
    };

    let mut osc = WaveformOscillator::new(config, 1000, Arc::clone(&wavetables));
    osc.change_freq(220);
}
