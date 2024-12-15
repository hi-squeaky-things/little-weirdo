use super::effects::filter::{FilterConfig, KindOfFilter};
use super::effects::overdrive::{KindOfOverdrive, OverdriveConfiguration};
use super::envelope::EnvelopConfiguration;
use super::mixer::MixerConfiguration;
use super::wavetable_oscillator::{WaveTableOscillatorConfig, Waveform};
use super::Patch;

#[derive(PartialEq, Clone, Copy)]
pub enum Patches {
    BassGuitar,
    ElectricPiano,
    GlidingBassGuitar,
    WeirdScience,
}

impl Patches {
    pub fn get_patch(name: Patches) -> Patch {
        match name {
            _ => Patch { 
                voices: [
                    WaveTableOscillatorConfig {
                        waveform: Waveform::Bass,
                        glide: false,
                        glide_rate: 0,
                        detune: 0,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        waveform: Waveform::Bass,
                        glide: false,
                        glide_rate: 0,
                        detune: 0,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        waveform: Waveform::Bass,
                        glide: false,
                        glide_rate: 0,
                        detune: 0,
                        freq: 440,
                    }, 
                ],
                envelops: [
                    EnvelopConfiguration { 
                        attack_time: 100, 
                        decay_time: 10, 
                        release_time: 100, 
                        sustain_level: 80 
                    }, 
                    EnvelopConfiguration { 
                        attack_time: 100, 
                        decay_time: 10, 
                        release_time: 100, 
                        sustain_level: 80 
                    }, 
                    EnvelopConfiguration { 
                        attack_time: 100, 
                        decay_time: 10, 
                        release_time: 100, 
                        sustain_level: 80 
                    }, 
                ],

                filter_config: FilterConfig { 
                    cutoff_frequency: 4_000, 
                    resonance: 40_000,
                    disabled: false, 
                    kind_of_filter: KindOfFilter::Low,
                }, 
                mixer_config: MixerConfiguration { 
                    gain_voice_1: 20, 
                    gain_voice_2: 20, 
                    gain_voice_3: 20,
                    gain_main: 50, 
                },
                overdrive_config: OverdriveConfiguration { 
                    threshold: 1000, 
                    kind: KindOfOverdrive::Softer, 
                    disabled: true, 
                }, 
            },
        }
    }
}

