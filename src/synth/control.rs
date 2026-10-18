//! Live parameter controls for [`super::Synth`].
//!
//! Audio rendering and note events remain on `Synth`; callers can use
//! [`Synth::control`](super::Synth::control) to access live adjustments through
//! this separate interface.

use super::{data::frequencies::MIDI2FREQ, Synth, AMOUNT_OF_VOICES};
use crate::{effects::{
    filter::KindOfFilter,
    overdrive::{KindOfOverdrive, OverdriveConfiguration},
}, synth::{noise::NoiseKind, patch::{Patch, SynthConfiguration}}};
extern crate alloc;
use alloc::string::String;

/// A live-control interface borrowing a synthesizer mutably.
///
/// Obtain this handle with [`Synth::control`](super::Synth::control). The
/// handle does not own the synthesizer and can be dropped before audio
/// rendering resumes.
pub struct SynthControl<'a> {
    synth: &'a mut Synth,
}

impl<'a> SynthControl<'a> {
    pub(super) fn new(synth: &'a mut Synth) -> Self {
        Self { synth }
    }

    pub fn get_current_patch(&mut self) -> Patch {
        Patch {
            name: self.synth.patch_name.clone(),
            voices: [
                self.synth.voices[0].config,
                self.synth.voices[1].config,
                self.synth.voices[2].config,
                self.synth.voices[3].config,
                self.synth.voices[4].config,
                self.synth.voices[5].config,
                self.synth.voices[6].config,
                self.synth.voices[7].config,
            ],
            envelops: [
                self.synth.envelops[0].configuration,
                self.synth.envelops[1].configuration,
                self.synth.envelops[2].configuration,
                self.synth.envelops[3].configuration,
                self.synth.envelops[4].configuration,
                self.synth.envelops[5].configuration,
                self.synth.envelops[6].configuration,
                self.synth.envelops[7].configuration,
            ],
            lfos: [
                self.synth.lfo[0].get_lfo_config(),
                self.synth.lfo[1].get_lfo_config(),
                self.synth.lfo[2].get_lfo_config(),
                self.synth.lfo[3].get_lfo_config(), 
            ],
            filter_config: self.synth.filter.config,
            mixer_config: self.synth.mixer.config,
            overdrive_config: self.synth.overdrive.config,
            bitcrunch_config: self.synth.bitcrunch.config,
            delay_config: self.synth.delay.config,
            routering_config: self.synth.router.config,
            synth_config: SynthConfiguration { mode: super::patch::SynthMode::OctoPoly },
            flanger_config: self.synth.flanger.config,
            fm_config: self.synth.fm_config,
        }
    }

    pub fn change_all_voices(&mut self, voice_id: u8) {
        for i in 0..AMOUNT_OF_VOICES {
            if self.synth.fm_config.enabled && i % 2 == 1 {
                continue;
            }
            let mut config = self.synth.voices[i].config;
            config.soundbank_index = voice_id;
            self.synth.voices[i].reload(config);
        }
    }

     pub fn change_all_voices_noise(&mut self, noise: NoiseKind) {
        for i in 0..AMOUNT_OF_VOICES {
            if self.synth.fm_config.enabled && i % 2 == 1 {
                continue;
            }
            let mut config = self.synth.voices[i].config;
            config.noise = noise;
            self.synth.voices[i].reload(config);
        }
    }


    
    pub fn change_all_voices_volume(&mut self, volume: u8)  {
        let mut config = self.synth.mixer.config;
         for i in 0..AMOUNT_OF_VOICES {
            config.gain_voices[i] = volume;
         }
          self.synth.mixer.reload(config);
    }



    /// Change the settings of all envelopes
    ///
    pub fn change_all_envelopes(
        &mut self,
        attack: Option<u16>,
        decay: Option<u16>,
        sustain: Option<i16>,
        release: Option<u16>,
    ) {

        let mut routing = self.synth.router.config;
          for i in 0..AMOUNT_OF_VOICES {
            routing.voices_to_envelop[i] = i as u8;
          }
          self.synth.router.reload(routing);

        for i in 0..AMOUNT_OF_VOICES {
            let mut config = self.synth.envelops[i].configuration;

            if let Some(val) = attack {
                config.attack_time = val;
            }
            if let Some(val) = decay {
                config.decay_time = val;
            }
            if let Some(val) = sustain {
                config.sustain_level = val;
            }
            if let Some(val) = release {
                config.release_time = val;
            }

            

            self.synth.envelops[i].update_configuration(config);
        }
    }

    pub fn overdrive_settings(&self) -> (bool, u8, u16) {
        let kind = match self.synth.overdrive.config.kind {
            KindOfOverdrive::Hard => 0,
            KindOfOverdrive::Soft => 1,
            KindOfOverdrive::Softer => 2,
        };
        (
            self.synth.overdrive.config.enabled,
            kind,
            self.synth.overdrive.config.threshold,
        )
    }

    pub fn toggle_overdrive(&mut self) {
        let mut config = self.synth.overdrive.config;
        config.enabled = !config.enabled;
        self.synth.overdrive.reload(config);
    }

    pub fn cycle_overdrive_kind(&mut self, clockwise: bool) {
        let mut config = self.synth.overdrive.config;
        config.kind = match (config.kind, clockwise) {
            (KindOfOverdrive::Hard, true) | (KindOfOverdrive::Softer, false) => {
                KindOfOverdrive::Soft
            }
            (KindOfOverdrive::Soft, true) => KindOfOverdrive::Softer,
            (KindOfOverdrive::Soft, false) => KindOfOverdrive::Hard,
            (KindOfOverdrive::Softer, true) => KindOfOverdrive::Hard,
            (KindOfOverdrive::Hard, false) => KindOfOverdrive::Softer,
        };
        self.synth.overdrive.reload(config);
    }

    pub fn adjust_overdrive_threshold(&mut self, clockwise: bool) {
        const THRESHOLD_STEP: u16 = 250;
        let mut config: OverdriveConfiguration = self.synth.overdrive.config;
        config.threshold = if clockwise {
            config
                .threshold
                .saturating_add(THRESHOLD_STEP)
                .min(i16::MAX as u16)
        } else {
            config.threshold.saturating_sub(THRESHOLD_STEP)
        };
        self.synth.overdrive.reload(config);
    }

    pub fn delay_settings(&self) -> (bool, u16, u8, u8) {
        (
            self.synth.delay.config.enabled,
            self.synth.delay.config.delay_time,
            self.synth.delay.config.mix_percentage,
            if self.synth.delay.config.feedback {
                self.synth.delay.config.feedback_percentage
            } else {
                0
            },
        )
    }

    pub fn flanger_settings(&self) -> (bool, u16, u16, u16, u8) {
        (
            self.synth.flanger.config.enabled,
            self.synth.flanger.config.base_delay_time_ms,
            self.synth.flanger.config.depth_ms,
            self.synth.flanger.config.rate_hz,
            self.synth.flanger.config.mix_percentage,
        )
    }

    pub fn filter_settings(&self) -> (bool, u16, u16, u8) {
        let kind = match self.synth.filter.config.kind_of_filter {
            KindOfFilter::Low => 0,
            KindOfFilter::High => 1,
            KindOfFilter::Band => 2,
            KindOfFilter::Notch => 3,
        };
        (
            self.synth.filter.config.enabled,
            self.synth.filter.config.cutoff_frequency,
            self.synth.filter.config.resonance,
            kind,
        )
    }

    pub fn toggle_filter(&mut self) {
        let mut config = self.synth.filter.config;
        config.enabled = !config.enabled;
        self.synth.filter.reload(config);
    }

    pub fn adjust_filter_cutoff(&mut self, clockwise: bool) {
        let mut config = self.synth.filter.config;
        config.cutoff_frequency = if clockwise {
            config.cutoff_frequency.saturating_add(500)
        } else {
            config.cutoff_frequency.saturating_sub(500)
        };
        self.synth.filter.reload(config);
    }

    pub fn adjust_filter_resonance(&mut self, clockwise: bool) {
        let mut config = self.synth.filter.config;
        config.resonance = if clockwise {
            config.resonance.saturating_add(500)
        } else {
            config.resonance.saturating_sub(500)
        };
        self.synth.filter.reload(config);
    }

    pub fn cycle_filter_kind(&mut self, clockwise: bool) {
        let mut config = self.synth.filter.config;
        let kind = match (config.kind_of_filter, clockwise) {
            (KindOfFilter::Low, true) => KindOfFilter::High,
            (KindOfFilter::Low, false) => KindOfFilter::Notch,
            (KindOfFilter::High, true) => KindOfFilter::Band,
            (KindOfFilter::High, false) => KindOfFilter::Low,
            (KindOfFilter::Band, true) => KindOfFilter::Notch,
            (KindOfFilter::Band, false) => KindOfFilter::High,
            (KindOfFilter::Notch, true) => KindOfFilter::Low,
            (KindOfFilter::Notch, false) => KindOfFilter::Band,
        };
        config.kind_of_filter = kind;
        self.synth.filter.reload(config);
    }

    pub fn toggle_flanger(&mut self) {
        let mut config = self.synth.flanger.config;
        config.enabled = !config.enabled;
        self.synth.flanger.reload(config, self.synth.sample_rate);
    }

    pub fn adjust_flanger_base_delay(&mut self, clockwise: bool) {
        let mut config = self.synth.flanger.config;
        config.base_delay_time_ms = if clockwise {
            config.base_delay_time_ms.saturating_add(1).min(2000)
        } else {
            config.base_delay_time_ms.saturating_sub(1)
        };
        self.synth.flanger.reload(config, self.synth.sample_rate);
    }

    pub fn adjust_flanger_depth(&mut self, clockwise: bool) {
        let mut config = self.synth.flanger.config;
        config.depth_ms = if clockwise {
            config.depth_ms.saturating_add(1).min(2000)
        } else {
            config.depth_ms.saturating_sub(1)
        };
        self.synth.flanger.reload(config, self.synth.sample_rate);
    }

    pub fn adjust_flanger_rate(&mut self, clockwise: bool) {
        let mut config = self.synth.flanger.config;
        config.rate_hz = if clockwise {
            config.rate_hz.saturating_add(1).min(100)
        } else {
            config.rate_hz.saturating_sub(1)
        };
        self.synth.flanger.reload(config, self.synth.sample_rate);
    }

    pub fn adjust_flanger_mix(&mut self, clockwise: bool) {
        let mut config = self.synth.flanger.config;
        config.mix_percentage = if clockwise {
            config.mix_percentage.saturating_add(1).min(100)
        } else {
            config.mix_percentage.saturating_sub(1)
        };
        self.synth.flanger.reload(config, self.synth.sample_rate);
    }

    pub fn fm_settings(&self) -> (bool, u8, u8, u16) {
        (
            self.synth.fm_config.enabled,
            self.synth.fm_config.modulator_waveform,
            self.synth.fm_config.modulator_ratio,
            self.synth.fm_config.modulation_depth_hz,
        )
    }

    pub fn set_fm_enabled(&mut self, enabled: bool) {
           self.synth.fm_config.enabled = enabled;
    }

    pub fn adjust_fm_modulator_waveform(&mut self, clockwise: bool) {
        if !self.synth.fm_config.enabled || self.synth.waveform_count == 0 {
            return;
        }

        let waveform_count = self.synth.waveform_count.min(u8::MAX as usize + 1);
        let current = self.synth.fm_config.modulator_waveform as usize % waveform_count;
        self.synth.fm_config.modulator_waveform = if clockwise {
            ((current + 1) % waveform_count) as u8
        } else {
            ((current + waveform_count - 1) % waveform_count) as u8
        };

        for index in (1..self.synth.voices.len()).step_by(2) {
            self.synth.voices[index].change_waveform(self.synth.fm_config.modulator_waveform);
        }
    }

    pub fn fm_carrier_waveform(&self) -> u8 {
        self.synth.fm_config.carrier_waveform
    }

    pub fn adjust_fm_carrier_waveform(&mut self, clockwise: bool) {
        if !self.synth.fm_config.enabled || self.synth.waveform_count == 0 {
            return;
        }

        let waveform_count = self.synth.waveform_count.min(u8::MAX as usize + 1);
        let current = self.synth.fm_config.carrier_waveform as usize % waveform_count;
        self.synth.fm_config.carrier_waveform = if clockwise {
            ((current + 1) % waveform_count) as u8
        } else {
            ((current + waveform_count - 1) % waveform_count) as u8
        };

        for index in (0..self.synth.voices.len()).step_by(2) {
            self.synth.voices[index].change_waveform(self.synth.fm_config.carrier_waveform);
        }
    }

    pub fn adjust_fm_modulator_ratio(&mut self, clockwise: bool) {
        if !self.synth.fm_config.enabled {
            return;
        }

        self.synth.fm_config.modulator_ratio = if clockwise {
            self.synth
                .fm_config
                .modulator_ratio
                .max(1)
                .saturating_add(1)
        } else {
            self.synth
                .fm_config
                .modulator_ratio
                .max(1)
                .saturating_sub(1)
                .max(1)
        };

        let voices_per_note = self.synth.voices_per_note();
        let max_active_notes = self.synth.max_active_notes();
        for (note_index, &note) in self
            .synth
            .active_note
            .iter()
            .take(max_active_notes)
            .enumerate()
        {
            if note == 0 {
                continue;
            }
            for pair_offset in (0..voices_per_note).step_by(2) {
                let carrier_index = note_index * voices_per_note + pair_offset;
                let modulator_index = carrier_index + 1;
                let freq = MIDI2FREQ
                    [(note as i8 + self.synth.voices[carrier_index].config.detune) as usize];
                let tuned_frequency = (freq as i32
                    + self.synth.voices[carrier_index].config.freq_detune as i32)
                    .clamp(0, u16::MAX as i32) as u16;
                let modulator_frequency = (tuned_frequency as u32
                    * self.synth.fm_config.modulator_ratio.max(1) as u32)
                    .min(u16::MAX as u32) as u16;
                self.synth.voices[modulator_index].change_freq(modulator_frequency);
            }
        }
    }

    pub fn adjust_fm_modulation_depth(&mut self, clockwise: bool) {
        if !self.synth.fm_config.enabled {
            return;
        }

        self.synth.fm_config.modulation_depth_hz = if clockwise {
            self.synth.fm_config.modulation_depth_hz.saturating_add(10)
        } else {
            self.synth.fm_config.modulation_depth_hz.saturating_sub(10)
        };
    }

    pub fn set_glide_enabled(&mut self, enabled: bool) {
        
    for i in 0..AMOUNT_OF_VOICES {
            if self.synth.fm_config.enabled && i % 2 == 1 {
                continue;
            }
            let mut config = self.synth.voices[i].config;
            config.glide = enabled;
            config.glide_rate = 10;
            self.synth.voices[i].reload(config);
        }
    }


    

      pub fn set_glide_time(&mut self, time: u8) {

        if time == 0 {
            return;
        }
        
    for i in 0..AMOUNT_OF_VOICES {
            if self.synth.fm_config.enabled && i % 2 == 1 {
                continue;
            }
            let mut config = self.synth.voices[i].config;
            config.glide_rate = time;
            self.synth.voices[i].reload(config);
        }
    }


    pub fn toggle_delay(&mut self) {
        let mut config = self.synth.delay.config;
        config.enabled = !config.enabled;
        self.synth.delay.reload(config, self.synth.sample_rate);
    }

    pub fn adjust_delay_time(&mut self, clockwise: bool) {
        const DELAY_TIME_STEP_MS: u16 = 10;
        let mut config = self.synth.delay.config;
        config.delay_time = if clockwise {
            config
                .delay_time
                .saturating_add(DELAY_TIME_STEP_MS)
                .min(5000)
        } else {
            config
                .delay_time
                .saturating_sub(DELAY_TIME_STEP_MS)
                .max(DELAY_TIME_STEP_MS)
        };
        self.synth.delay.reload(config, self.synth.sample_rate);
    }

    pub fn adjust_delay_mix(&mut self, clockwise: bool) {
        let mut config = self.synth.delay.config;
        config.mix_percentage = if clockwise {
            config.mix_percentage.saturating_add(1).min(100)
        } else {
            config.mix_percentage.saturating_sub(1)
        };
        self.synth.delay.reload(config, self.synth.sample_rate);
    }

    pub fn adjust_delay_feedback(&mut self, clockwise: bool) {
        let mut config = self.synth.delay.config;
        let current_feedback = if config.feedback {
            config.feedback_percentage
        } else {
            0
        };
        config.feedback_percentage = if clockwise {
            current_feedback.saturating_add(1).min(100)
        } else {
            current_feedback.saturating_sub(1)
        };
        config.feedback = config.feedback_percentage != 0;
        self.synth.delay.reload(config, self.synth.sample_rate);
    }

    /// Change the main volume of the synthesizer
    pub fn change_main_volume(&mut self, velocity: u8) {
        self.synth.mixer.config.gain_main = velocity;
    }


    /// Change the filter cutoff frequency
    pub fn change_cutoff(&mut self, velocity: u8) {
        let mut config = self.synth.filter.config;
        config.cutoff_frequency = velocity as u16 * 255;
        self.synth.filter.reload(config);
    }

    /// Change the filter resonance
    pub fn change_resonance(&mut self, velocity: u8) {
        let mut config = self.synth.filter.config;
        config.resonance = velocity as u16 * 255;
        self.synth.filter.reload(config);
    }
}
