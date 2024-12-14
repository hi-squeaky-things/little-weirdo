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
                voice_1: WaveTableOscillatorConfig {
                    waveform: Waveform::EightBit,
                    glide: false,
                    glide_rate: 0,
                    detune: 0,
                    freq: 440,
                }, 
                voice_1_env: EnvelopConfiguration { 
                    attack_time: 10, 
                    decay_time: 10, 
                    release_time: 100, 
                    sustain_level: 80 
                }, 
                voice_2: WaveTableOscillatorConfig {
                    waveform: Waveform::SawTooth,
                    glide: false,
                    glide_rate: 0,
                    detune: 4,
                    freq: 440,
                },  
                voice_3_env: EnvelopConfiguration { 
                    attack_time: 10, 
                    decay_time: 10, 
                    release_time: 100, 
                    sustain_level: 80,
                },
                voice_3: WaveTableOscillatorConfig {
                    waveform: Waveform::Square10,
                    glide: false,
                    glide_rate: 0,
                    detune: 7,
                    freq: 440,
                },  
                voice_2_env: EnvelopConfiguration { 
                    attack_time: 10, 
                    decay_time: 10, 
                    release_time: 100, 
                    sustain_level: 80,
                },
                filter_config: FilterConfig { 
                    cutoff_frequency: 2000, 
                    pass_through: true, 
                    kind_of_filter: KindOfFilter::Low
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
                    pass_through: true, 
                }, 
            },
        }
    }
}

