use super::effects::filter::{FilterConfig, KindOfFilter};
use super::effects::overdrive::{KindOfOverdrive, OverdriveConfiguration};
use super::envelope::EnvelopConfiguration;
use super::mixer::MixerConfiguration;
use super::wavetable_oscillator::{WaveTableOscillatorConfig, Waveform};
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
                    gain_voices: [
                        20,
                        0,
                        0,
                        0,
                    ],
                    gain_main: 50, 
                },
                overdrive_config: OverdriveConfiguration { 
                    threshold: 1000, 
                    kind: KindOfOverdrive::Softer, 
                    disabled: true, 
                }, 
            },
            Patches::BassGuitarFourth => Patch { 
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
                        detune: 4,
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
                    EnvelopConfiguration { 
                        attack_time: 100, 
                        decay_time: 10, 
                        release_time: 100, 
                        sustain_level: 80 
                    }, 
                ],

                filter_config: FilterConfig { 
                    cutoff_frequency: 1_000, 
                    resonance: 30_000,
                    disabled: false, 
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
                    disabled: true, 
                }, 
            },
            Patches::BassGuitarMajor => Patch { 
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
                        detune: 4,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        waveform: Waveform::Bass,
                        glide: false,
                        glide_rate: 0,
                        detune: 7,
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
                    gain_voices: [
                        20,
                        20,
                        20,
                        0,
                    ],
                    gain_main: 50, 
                },
                overdrive_config: OverdriveConfiguration { 
                    threshold: 1000, 
                    kind: KindOfOverdrive::Softer, 
                    disabled: true, 
                }, 
            },
            Patches::WeirdScience => Patch { 
                voices: [
                    WaveTableOscillatorConfig {
                        waveform: Waveform::Square,
                        glide: false,
                        glide_rate: 0,
                        detune: 0,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        waveform: Waveform::Square,
                        glide: false,
                        glide_rate: 0,
                        detune: 4,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        waveform: Waveform::Square,
                        glide: false,
                        glide_rate: 0,
                        detune: 7,
                        freq: 440,
                    }, 
                    WaveTableOscillatorConfig {
                        waveform: Waveform::Square,
                        glide: false,
                        glide_rate: 0,
                        detune: 9,
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

                filter_config: FilterConfig { 
                    cutoff_frequency: 7_000, 
                    resonance: 0,
                    disabled: false, 
                    kind_of_filter: KindOfFilter::Low,
                }, 
                mixer_config: MixerConfiguration { 
                    gain_voices: [
                        20,
                        20,
                        20,
                        20,
                    ],
                    gain_main: 50, 
                },
                overdrive_config: OverdriveConfiguration { 
                    threshold: 1000, 
                    kind: KindOfOverdrive::Soft, 
                    disabled: true, 
                }, 
            },
        }
    }
}

