use super::oscillator::Waveform;
use super::Patch;
use super::envelope::Envelop;
use super::filter::FilterConfig;

#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub enum Patches {
    bass_guitar,
    eletric_piano,
    gliding_bass_guitar,
    hihat,
}

impl Patches {
    pub fn get_patch(name: Patches) -> Patch {
       match name {
        Patches::bass_guitar => Patch {
                voice_1: Waveform::Bass,
                voice_1_mix_level: 100,
                voice_2: Waveform::Bass,
                voice_2_mix_level: 0,
                lfo_1: 20,
                lfo_1_mix_level: 0,
                voice_1_env: Envelop {
                    attack_time: 10,
                    decay_time: 10,
                    release_time: 200,
                    sustain_level: 80,
                },
                voice_2_env: Envelop {
                    attack_time: 500,
                    decay_time: 150,
                    release_time: 150,
                    sustain_level: 50,
                },
                voice_1_detune: 0,
                voice_2_detune: 14,
                filter_config: FilterConfig {
                    cutoff_frequency: 100,
                    filter_on: true,
                    _24db: false,
                },
                main_gain: 50,
                glide: false,
                glide_rate: 30,
                mono: true,
            },
            Patches::eletric_piano => Patch {
                voice_1: Waveform::Piano,
                voice_1_mix_level: 50,
                voice_2: Waveform::Piano,
                voice_2_mix_level: 0,
                lfo_1: 20,
                lfo_1_mix_level: 0,
                voice_1_env: Envelop {
                    attack_time: 500,
                    decay_time: 150,
                    release_time: 150,
                    sustain_level: 50,
                },
                voice_2_env: Envelop {
                    attack_time: 500,
                    decay_time: 150,
                    release_time: 150,
                    sustain_level: 50,
                },
                voice_1_detune: 0,
                voice_2_detune: 0,
                filter_config: FilterConfig {
                    cutoff_frequency: 400,
                    filter_on: false,
                    _24db: false,
                },
                main_gain: 50,
                glide: false,
                glide_rate: 30,
                mono: true,
            },
            Patches::gliding_bass_guitar => Patch {
                voice_1: Waveform::Bass,
                voice_1_mix_level: 100,
                voice_2: Waveform::Bass,
                voice_2_mix_level: 0,
                lfo_1: 20,
                lfo_1_mix_level: 0,
                voice_1_env: Envelop {
                    attack_time: 10,
                    decay_time: 10,
                    release_time: 200,
                    sustain_level: 80,
                },
                voice_2_env: Envelop {
                    attack_time: 500,
                    decay_time: 150,
                    release_time: 150,
                    sustain_level: 50,
                },
                voice_1_detune: 0,
                voice_2_detune: 14,
                filter_config: FilterConfig {
                    cutoff_frequency: 100,
                    filter_on: true,
                    _24db: false,
                },
                main_gain: 50,
                glide: true,
                glide_rate: 30,
                mono: true,
            },
           Patches::hihat => Patch {
                voice_1: Waveform::EightBit,
                voice_1_mix_level: 50,
                voice_2: Waveform::Square10,
                voice_2_mix_level: 50,
                lfo_1: 20,
                lfo_1_mix_level: 0,
                voice_1_env: Envelop {
                    attack_time: 10,
                    decay_time: 10,
                    release_time: 10,
                    sustain_level: 80,
                },
                voice_2_env: Envelop {
                    attack_time: 10,
                    decay_time: 10,
                    release_time: 10,
                    sustain_level: 80,
                },
                voice_1_detune: 0,
                voice_2_detune: 4,
                filter_config: FilterConfig {
                    cutoff_frequency: 100,
                    filter_on: false,
                    _24db: false,
                },
                main_gain: 50,
                glide: false,
                glide_rate: 30,
                mono: true,
            },
        }
    }
}


 