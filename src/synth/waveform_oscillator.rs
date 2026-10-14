//! Oscillator to generate waveforms based on waveform samples.

use crate::synth::data::waveforms::BoxedWaveforms;

use super::data::waveforms::Waveforms;
use super::math::percentage;
use super::noise::{NoiseGenerator, NoiseKind};
use super::Clockable;

use serde::Deserialize;
use serde::Serialize;

extern crate alloc;
use alloc::sync::Arc;

const PHASE_SCALE: u64 = 1 << 32;
const FREQUENCY_SCALE: u16 = 5;

fn effective_noise_kind(config: WaveformOscillatorConfig) -> NoiseKind {
    match config.noise {
        NoiseKind::None if config.soundbank_index == 255 => NoiseKind::White,
        kind => kind,
    }
}

/// Patch-compatible configuration for a low-frequency waveform oscillator.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct WaveformLFOConfig {
    /// Index of the soundbank to use for waveform generation
    pub soundbank_index: u8,
    /// LFO period in hundredths of a second; zero disables the LFO
    pub time: u16,
}

/// Configuration for standard waveform oscillator
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct WaveformOscillatorConfig {
    /// Index of the soundbank to use for waveform generation
    pub soundbank_index: u8,
    /// Whether to enable frequency glide (smooth frequency transitions)
    pub glide: bool,
    /// Rate of frequency glide (0 to 255, higher values = faster glide)
    pub glide_rate: u8,
    /// Detune amount in semitones (-128 to 127)
    pub detune: i8,
    /// Frequency detune amount in Hz (-128 to 127)
    pub freq_detune: i8,
    // Use a grain oscillator.
    pub grains: bool,
    // List of grains used
    pub grains_seq: [u8; 8],
    /// Noise source to use instead of the wavetable.
    #[serde(default)]
    pub noise: NoiseKind,
}

/// WaveformOscillator generates audio signals using waveform (sample) synthesis.
pub struct WaveformOscillator {
    /// Configuration settings for this oscillator instance
    pub config: WaveformOscillatorConfig,

    /// Fractional position within the waveform cycle, represented as Q0.32.
    phase: u32,

    /// Phase advance per processed sample, represented as Q0.32.
    phase_increment: u64,

    /// Flag indicating whether frequency has changed and needs processing
    freq_changed: bool,

    /// Integer noise generator for noise voices
    noise_generator: NoiseGenerator,

    /// Sample rate of the audio system (used for frequency calculations)
    sample_rate: u16,

    /// Target frequency for glide operations
    target_freq: u16,

    /// Original frequency before any detuning
    original_freq: u16,

    /// Step size for frequency glide calculations
    freq_step: i32,

    /// Current actual frequency being used for playback
    current_freq: u16,

    /// Whether this oscillator is producing samples
    enabled: bool,

    /// Shared reference to the waveforms data structure
    waveforms: Arc<BoxedWaveforms>,

    /// Waveform pointer, points to the grain being selected
    waveform_pointer: u8,

    grain_selector: u8,
}

impl Clockable for WaveformOscillator {
    fn clock(&mut self, _sample: Option<i16>) -> i16 {
        if !self.enabled {
            return 0;
        }

        // Generate output
        let noise_kind = effective_noise_kind(self.config);
        let output = match noise_kind {
            NoiseKind::None => {
                let waveform = self.waveforms.get_waveform_reference(self.waveform_pointer);
                let index = ((self.phase as u64 * waveform.len() as u64) >> 32) as usize;
                waveform[index]
            }
            _ => self.noise_generator.sample(),
        };

        let next_phase = self.phase as u64 + self.phase_increment;
        let completed_cycles = next_phase >> 32;
        self.phase = next_phase as u32;

        if completed_cycles > 0 {
            if self.config.grains {
                self.grain_selector = (self.grain_selector as u64 + completed_cycles) as u8 & 0x7;
                self.waveform_pointer = self.config.grains_seq[self.grain_selector as usize];

                if self.waveform_pointer == 255 {
                    self.grain_selector = 0;
                    self.waveform_pointer = self.config.grains_seq[self.grain_selector as usize];
                }
            }

            if self.freq_changed {
                if self.current_freq != self.target_freq {
                    let diff = self.target_freq as i32 - self.current_freq as i32;
                    let step_size = self.freq_step;

                    if diff.abs() < step_size.abs() {
                        self.current_freq = self.target_freq;
                        self.freq_changed = false;
                    } else {
                        self.current_freq = (self.current_freq as i32 + step_size) as u16;
                    }
                } else {
                    self.freq_changed = false;
                }

                self.phase_increment = Self::phase_increment(self.sample_rate, self.current_freq);
            }
        }

        output
    }
}

impl WaveformOscillator {
    /// Creates an oscillator using the patch-compatible LFO period setting.
    pub fn new_lfo(
        config: WaveformLFOConfig,
        sample_rate: u16,
        waveforms: Arc<BoxedWaveforms>,
    ) -> Self {
        let new_config = WaveformOscillatorConfig {
            soundbank_index: config.soundbank_index,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq_detune: 0,
            grains: false,
            grains_seq: [0; 8],
            noise: NoiseKind::None,
        };
        let mut osc = Self::new(new_config, sample_rate, waveforms);
        osc.enabled = config.time != 0;
        osc.phase_increment = Self::phase_increment_for_period(sample_rate, config.time);
        osc
    }

    pub fn new(
        config: WaveformOscillatorConfig,
        sample_rate: u16,
        waveforms: Arc<BoxedWaveforms>,
    ) -> Self {
        Self::new_with_seed(config, sample_rate, waveforms, 23702372039u64)
    }

    pub fn new_with_seed(
        config: WaveformOscillatorConfig,
        sample_rate: u16,
        waveforms: Arc<BoxedWaveforms>,
        noise_seed: u64,
    ) -> Self {
        let mut osc = Self {
            config,
            phase: 0,
            phase_increment: Self::phase_increment(sample_rate, 440 * FREQUENCY_SCALE),
            freq_changed: false,
            noise_generator: NoiseGenerator::new(effective_noise_kind(config), noise_seed),
            sample_rate,
            target_freq: 440 * FREQUENCY_SCALE,
            original_freq: 440 * FREQUENCY_SCALE,
            current_freq: 440 * FREQUENCY_SCALE,
            freq_step: 0,
            enabled: true,
            waveforms,
            waveform_pointer: config.soundbank_index,
            grain_selector: 0,
        };
        if config.grains {
            osc.waveform_pointer = config.grains_seq[osc.grain_selector as usize];
        }
        osc
    }

    fn phase_increment(sample_rate: u16, frequency: u16) -> u64 {
        if sample_rate == 0 || frequency == 0 {
            return 0;
        }

        let scaled_sample_rate = sample_rate as u64 * FREQUENCY_SCALE as u64;
        ((frequency as u64 * PHASE_SCALE) + scaled_sample_rate / 2) / scaled_sample_rate
    }

    fn phase_increment_for_period(sample_rate: u16, period_hundredths: u16) -> u64 {
        if sample_rate == 0 || period_hundredths == 0 {
            return 0;
        }

        let denominator = sample_rate as u64 * period_hundredths as u64;
        (PHASE_SCALE * 100 + denominator / 2) / denominator
    }

    /// Reset internal state to default values
    fn reset_state(&mut self) {
        self.phase = 0;
        self.phase_increment = Self::phase_increment(self.sample_rate, 440 * FREQUENCY_SCALE);
        self.freq_changed = false;
        self.target_freq = 440 * FREQUENCY_SCALE;
        self.original_freq = 440 * FREQUENCY_SCALE;
        self.current_freq = 440 * FREQUENCY_SCALE;
        self.freq_step = 0;
        self.enabled = true;
        self.grain_selector = 0;
        self.noise_generator
            .set_kind(effective_noise_kind(self.config));
    }

    /// Reload configuration
    pub fn reload(&mut self, config: WaveformOscillatorConfig) {
        self.config = config;
        self.reset_state();
        self.waveform_pointer = config.soundbank_index;

        if self.config.grains {
            self.waveform_pointer = config.grains_seq[self.grain_selector as usize];
        }
    }

    pub fn reload_lfo(&mut self, config: WaveformLFOConfig) {
        let new_config = WaveformOscillatorConfig {
            soundbank_index: config.soundbank_index,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq_detune: 0,
            grains: false,
            grains_seq: [0; 8],
            noise: NoiseKind::None,
        };
        self.reset_state();
        self.enabled = config.time != 0;
        self.phase_increment = Self::phase_increment_for_period(self.sample_rate, config.time);
        self.waveform_pointer = config.soundbank_index;
        self.config = new_config;
    }

    /// Manipulate frequency with percentage
    pub fn manipulate_freq(&mut self, perc: u8, amount: u16) {
        let modulation = percentage(amount as i16, perc as i16) as i32 * FREQUENCY_SCALE as i32;
        self.current_freq = (self.original_freq as i32 + modulation) as u16;
        self.freq_changed = true;
    }

    /// Change frequency in fifths of a hertz, with optional glide.
    pub fn change_freq(&mut self, frequency: u16) {
        let frequency = (frequency as i32
            + self.config.freq_detune as i32 * (FREQUENCY_SCALE as i32 - 1))
            as u16;
        if self.current_freq != frequency {
            self.target_freq = frequency;

            if self.config.glide {
                let diff = frequency as i32 - self.current_freq as i32;
                self.freq_step = diff / self.config.glide_rate as i32;
                // Ensure non-zero step
                if self.freq_step == 0 {
                    self.freq_step = diff / 2;
                }
            } else {
                self.current_freq = frequency;
            }

            self.original_freq = frequency;
            self.freq_changed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::data::frequencies::MIDI2FREQ;
    use crate::synth::data::waveforms::{BoxedWaveform, BoxedWaveforms};

    fn test_waveforms() -> Arc<BoxedWaveforms> {
        let mut waveforms = BoxedWaveforms::new();
        waveforms.add(BoxedWaveform::new(&[0; 1200]));
        Arc::new(waveforms)
    }

    fn lfo_config(time: u16) -> WaveformLFOConfig {
        WaveformLFOConfig {
            soundbank_index: 0,
            time,
        }
    }

    #[test]
    fn phase_accumulator_tracks_note_frequency_without_integer_period_rounding() {
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
        let mut oscillator = WaveformOscillator::new(config, 44_100, test_waveforms());
        let frequency = MIDI2FREQ[76];
        assert_eq!(frequency, 3296);
        oscillator.current_freq = frequency;
        oscillator.phase_increment = WaveformOscillator::phase_increment(44_100, frequency);

        let mut completed_cycles = 0;
        for _ in 0..44_100 {
            let previous_phase = oscillator.phase;
            oscillator.clock(None);
            if oscillator.phase < previous_phase {
                completed_cycles += 1;
            }
        }

        assert!((658..=660).contains(&completed_cycles));
    }

    #[test]
    fn lfo_period_is_measured_in_hundredths_of_a_second() {
        let mut oscillator = WaveformOscillator::new_lfo(lfo_config(200), 44_100, test_waveforms());
        assert_eq!(
            oscillator.phase_increment,
            WaveformOscillator::phase_increment_for_period(44_100, 200)
        );

        for _ in 0..44_100 {
            oscillator.clock(None);
        }

        assert!(
            (PHASE_SCALE as i64 / 2 - 22_050..=PHASE_SCALE as i64 / 2 + 22_050)
                .contains(&(oscillator.phase as i64))
        );
    }

    #[test]
    fn lfo_supports_periods_of_several_seconds() {
        let mut oscillator = WaveformOscillator::new_lfo(lfo_config(500), 44_100, test_waveforms());

        for _ in 0..220_500 {
            oscillator.clock(None);
        }

        assert!(oscillator.phase > u32::MAX - 100_000);
    }

    #[test]
    fn reloading_lfo_updates_its_period_and_disabled_state() {
        let mut oscillator = WaveformOscillator::new_lfo(lfo_config(100), 44_100, test_waveforms());
        for _ in 0..100 {
            oscillator.clock(None);
        }
        assert_ne!(oscillator.phase, 0);

        oscillator.reload_lfo(lfo_config(500));
        assert_eq!(oscillator.phase, 0);
        assert_eq!(
            oscillator.phase_increment,
            WaveformOscillator::phase_increment_for_period(44_100, 500)
        );

        oscillator.reload_lfo(lfo_config(0));
        assert!(!oscillator.enabled);
        assert_eq!(oscillator.clock(None), 0);
    }

    #[test]
    fn oscillator_supports_high_audio_frequencies() {
        let frequency = u16::MAX;
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
        let mut oscillator = WaveformOscillator::new(config, 44_100, test_waveforms());
        oscillator.current_freq = frequency;
        oscillator.phase_increment = WaveformOscillator::phase_increment(44_100, frequency);
        let mut completed_cycles = 0;

        for _ in 0..44_100 {
            let previous_phase = oscillator.phase;
            oscillator.clock(None);
            if oscillator.phase < previous_phase {
                completed_cycles += 1;
            }
        }

        assert!((13_106..=13_108).contains(&completed_cycles));
    }

    #[test]
    fn zero_lfo_period_stays_disabled() {
        let mut oscillator = WaveformOscillator::new_lfo(lfo_config(0), 44_100, test_waveforms());

        for _ in 0..100 {
            assert_eq!(oscillator.clock(None), 0);
        }

        assert_eq!(oscillator.phase, 0);
    }
}
