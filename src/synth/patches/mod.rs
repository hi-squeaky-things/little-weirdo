use super::effects::filter::{FilterConfig, KindOfFilter};
use super::effects::overdrive::{KindOfOverdrive, OverdriveConfiguration};
use super::envelope::EnvelopConfiguration;
use super::mixer::MixerConfiguration;
use super::router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute};
use super::wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig};
use super::Patch;

#[derive(PartialEq, Clone, Copy)]
pub enum Patches {
    BassGuitar,
    BassGuitarFourth,
    BassGuitarMajor,
    WeirdScience,
}

impl Patches {
    pub fn get_patch(name: Patches) -> Patch {
        match name {
            Patches::BassGuitar => Patch { 
                voices: [
                    WaveTableOscillatorConfig {
                        soundbank_index: 6,
                        glide: false,
                        glide_rate: 10,
                        detune: 0,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 7,
                        glide: false,
                        glide_rate: 0,
                        detune: 4,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 0,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
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
                    EnvelopConfiguration { 
                        attack_time: 100, 
                        decay_time: 10, 
                        release_time: 100, 
                        sustain_level: 80 
                    }, 
                ],
                lfo: WaveTableLoFreqOscillatorConfig {
                     soundbank_index: 0,
                    time: 5,
                },
                routering_config: RoutingConfiguration {
                    voices_to_envelop: [
                        VoiceToEnvelopRoute {
                            env: 0,
                        },
                        VoiceToEnvelopRoute {
                            env: 1,
                        },  
                        VoiceToEnvelopRoute {
                            env: 2,
                        },  
                        VoiceToEnvelopRoute {
                            env: 3,
                        },  
                        ],
                    voice_to_lfo : VoiceToLFORoute {
                        enable: false,
                        voice: 0,
                    },
                    lfo_to_filter: false,
                },   
                filter_config: FilterConfig { 
                    cutoff_frequency: 1_000, 
                    resonance: 5_000,
                    enabled: false, 
                    kind_of_filter: KindOfFilter::Low,
                }, 
                mixer_config: MixerConfiguration { 
                    gain_voices: [
                        50,
                        0,
                        0,
                        0,
                    ],
                    gain_main: 50, 
                },
                overdrive_config: OverdriveConfiguration { 
                    threshold: 2000, 
                    kind: KindOfOverdrive::Softer, 
                    enabled: false, 
                }, 
            },
            Patches::BassGuitarFourth => Patch { 
                voices: [
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 0,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 4,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 0,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
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
                    EnvelopConfiguration { 
                        attack_time: 100, 
                        decay_time: 10, 
                        release_time: 100, 
                        sustain_level: 80 
                    }, 
                ],
                lfo: WaveTableLoFreqOscillatorConfig {
                     soundbank_index: 0,
                    time: 10,
                },
                routering_config: RoutingConfiguration {
                    voices_to_envelop: [
                        VoiceToEnvelopRoute {
                            env: 0,
                        },
                        VoiceToEnvelopRoute {
                            env: 1,
                        },  
                        VoiceToEnvelopRoute {
                            env: 2,
                        },  
                        VoiceToEnvelopRoute {
                            env: 3,
                        },  
                        ],
                        voice_to_lfo : VoiceToLFORoute {
                            enable: true,
                            voice: 0,
                        },
                        lfo_to_filter: false,
                },   
                filter_config: FilterConfig { 
                    cutoff_frequency: 1_000, 
                    resonance: 0,
                    enabled: true, 
                    kind_of_filter: KindOfFilter::Low,
                }, 
                mixer_config: MixerConfiguration { 
                    gain_voices: [
                        50,
                        50,
                        0,
                        0,
                    ],
                    gain_main: 100, 
                },
                overdrive_config: OverdriveConfiguration { 
                    threshold: 1000, 
                    kind: KindOfOverdrive::Softer, 
                    enabled: true, 
                }, 
            },
            Patches::BassGuitarMajor => Patch { 
                voices: [
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 0,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 4,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 7,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
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
                    EnvelopConfiguration { 
                        attack_time: 100, 
                        decay_time: 10, 
                        release_time: 100, 
                        sustain_level: 80 
                    }, 
                ],
                lfo: WaveTableLoFreqOscillatorConfig {
                     soundbank_index: 0,
                    time: 10,
                },
                routering_config: RoutingConfiguration {
                    voices_to_envelop: [
                        VoiceToEnvelopRoute {
                            env: 0,
                        },
                        VoiceToEnvelopRoute {
                            env: 1,
                        },  
                        VoiceToEnvelopRoute {
                            env: 2,
                        },  
                        VoiceToEnvelopRoute {
                            env: 3,
                        },  
                        ],
                        voice_to_lfo : VoiceToLFORoute {
                            enable: false,
                            voice: 0,
                        },
                        lfo_to_filter: false,

                },   
                filter_config: FilterConfig { 
                    cutoff_frequency: 1_000, 
                    resonance: 0,
                    enabled: true, 
                    kind_of_filter: KindOfFilter::Low,
                }, 
                mixer_config: MixerConfiguration { 
                    gain_voices: [
                        25,
                        25,
                        25,
                        0,
                    ],
                    gain_main: 50, 
                },
                overdrive_config: OverdriveConfiguration { 
                    threshold: 1000, 
                    kind: KindOfOverdrive::Softer, 
                    enabled: false, 
                }, 
            },
            Patches::WeirdScience => Patch { 
                voices: [
                    WaveTableOscillatorConfig {
                         soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 0,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 4,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 7,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        soundbank_index: 0,
                        glide: false,
                        glide_rate: 0,
                        detune: 9,
                        freq: 440,
                    }, 
                ],
                envelops: [
                    EnvelopConfiguration { 
                        attack_time: 2000, 
                        decay_time: 50, 
                        release_time: 1000, 
                        sustain_level: 50 
                    }, 
                    EnvelopConfiguration { 
                        attack_time: 100, 
                        decay_time: 10, 
                        release_time: 100, 
                        sustain_level: 50 
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
                lfo: WaveTableLoFreqOscillatorConfig {
                     soundbank_index: 0,
                    time: 1,
                },
                routering_config: RoutingConfiguration {
                    voices_to_envelop: [
                        VoiceToEnvelopRoute {
                            env: 0,
                        },
                        VoiceToEnvelopRoute {
                            env: 0,
                        },  
                        VoiceToEnvelopRoute {
                            env: 0,
                        },  
                        VoiceToEnvelopRoute {
                            env: 0,
                        },  
                        ],
                        voice_to_lfo : VoiceToLFORoute {
                            enable: false,
                            voice: 0,
                        },
                        lfo_to_filter: false,
                },   
                filter_config: FilterConfig { 
                    cutoff_frequency: 10_000, 
                    resonance: 0,
                    enabled: true, 
                    kind_of_filter: KindOfFilter::High,
                }, 
                mixer_config: MixerConfiguration { 
                    gain_voices: [
                        100,
                        0,
                        0,
                        0,
                    ],
                    gain_main: 100, 
                },
                overdrive_config: OverdriveConfiguration { 
                    threshold: 1000, 
                    kind: KindOfOverdrive::Soft, 
                    enabled: true, 
                }, 
            },
        }
    }
}

