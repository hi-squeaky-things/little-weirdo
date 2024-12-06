use super::effects::filter::FilterConfig;
use super::effects::overdrive::KindOfOverdrive;
use super::envelope::Envelop;
use super::oscillator::Waveform;
use super::Patch;

#[derive(PartialEq, Clone, Copy)]
pub enum Patches {
    BassGuitar,
    EletricPiano,
    GlidingBassGuitar,
    Hihat,
}

impl Patches {
    pub fn get_patch(name: Patches) -> Patch {
        match name {
            Patches::BassGuitar => Patch {
                voice_1: Waveform::Bass,
                voice_1_mix_level: 100,
                voice_2: Waveform::EightBit,
                voice_2_mix_level: 0,
                lfo_1: 20,
                lfo_1_mix_level: 0,
                voice_1_env: Envelop {
                    attack_time: 100,
                    decay_time: 100,
                    release_time: 100,
                    sustain_level: 50,
                },
                voice_2_env: Envelop {
                    attack_time: 10,
                    decay_time: 10,
                    release_time: 200,
                    sustain_level: 80,
                },
                voice_1_detune: 0,
                voice_2_detune: 4,
                filter_config: FilterConfig {
                    cutoff_frequency: 500,
                    filter_on: false,
                },
                main_gain: 80,
                glide: false,
                glide_rate: 30,
                mono: true,
                overdrive: false,
                overdrive_mode: KindOfOverdrive::Hard,
            },
            Patches::EletricPiano => Patch {
                voice_1: Waveform::Piano,
                voice_1_mix_level: 100,
                voice_2: Waveform::EightBit,
                voice_2_mix_level: 10,
                lfo_1: 20,
                lfo_1_mix_level: 0,
                voice_1_env: Envelop {
                    attack_time: 20,
                    decay_time: 20,
                    release_time: 500,
                    sustain_level: 80,
                },
                voice_2_env: Envelop {
                    attack_time: 20,
                    decay_time: 20,
                    release_time: 500,
                    sustain_level: 80,
                },
                voice_1_detune: 0,
                voice_2_detune: 0,
                filter_config: FilterConfig {
                    cutoff_frequency: 400,
                    filter_on: false,
                },
                main_gain: 50,
                glide: false,
                glide_rate: 30,
                mono: true,
                overdrive: false,
                overdrive_mode: KindOfOverdrive::Soft,
            },
            Patches::GlidingBassGuitar => Patch {
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
                },
                main_gain: 50,
                glide: true,
                glide_rate: 30,
                mono: true,
                overdrive: false,
                overdrive_mode: KindOfOverdrive::Hard,
            },
            Patches::Hihat => Patch {
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
                },
                main_gain: 50,
                glide: false,
                glide_rate: 30,
                mono: true,
                overdrive: false,
                overdrive_mode: KindOfOverdrive::Hard,
            },
        }
    }
}
