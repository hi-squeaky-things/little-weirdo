use super::effects::filter::FilterConfig;
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
                    waveform: Waveform::Piano,
                    glide: false,
                    glide_rate: 10,
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
                    waveform: Waveform::Piano,
                    glide: false,
                    glide_rate: 0,
                    detune: 2,
                    freq: 440,
                },  
                voice_3_env: EnvelopConfiguration { 
                    attack_time: 10, 
                    decay_time: 10, 
                    release_time: 100, 
                    sustain_level: 80,
                },
                voice_3: WaveTableOscillatorConfig {
                    waveform: Waveform::Piano,
                    glide: false,
                    glide_rate: 0,
                    detune: 4,
                    freq: 440,
                },  
                voice_2_env: EnvelopConfiguration { 
                    attack_time: 10, 
                    decay_time: 10, 
                    release_time: 100, 
                    sustain_level: 80,
                },
                filter_config: FilterConfig { 
                    cutoff_frequency: 440, 
                    filter_on: false, 
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
                mono: true 
            },
        }
    }
}

