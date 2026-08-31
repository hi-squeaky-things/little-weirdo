use std::{fs, sync::Arc};

use little_weirdo::synth::{
    data::wavetables::{BoxedWavetable, BoxedWavetables},
    wavetable_oscillator::{WaveTableOscillator, WaveTableOscillatorConfig},
};

fn load_sine_wavetable() -> Arc<BoxedWavetables> {
    let raw = fs::read("examples/soundbank/synth/src/0_sample.raw").unwrap();
    let mut wavetables = BoxedWavetables::new();
    wavetables.add(BoxedWavetable::new(&raw));
    Arc::new(wavetables)
}

#[test]
fn wavetable_oscillator_outputs_samples_from_the_selected_table() {
    let wavetables = load_sine_wavetable();
    let config = WaveTableOscillatorConfig {
        soundbank_index: 0,
        glide: false,
        glide_rate: 0,
        detune: 0,
        freq_detune: 0,
        grains: false,
        grains_seq: [0; 8],
    };

    let _osc = WaveTableOscillator::new(config, 1000, Arc::clone(&wavetables));


}

#[test]
fn wavetable_oscillator_can_change_frequency() {
    let wavetables = load_sine_wavetable();
    let config = WaveTableOscillatorConfig {
        soundbank_index: 0,
        glide: false,
        glide_rate: 0,
        detune: 0,
        freq_detune: 0,
        grains: false,
        grains_seq: [0; 8],
    };

    let mut osc = WaveTableOscillator::new(config, 1000, Arc::clone(&wavetables));
    osc.change_freq(220);

}
