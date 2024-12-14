//! Patch to config the sounds
 
use super::{effects::{filter::FilterConfig, overdrive::OverdriveConfiguration}, envelope::EnvelopConfiguration, mixer::MixerConfiguration, patches::Patches, wavetable_oscillator::WaveTableOscillatorConfig};

pub struct Patch {
    /// The waveform for voice one, this is one of the available WaveTable waveforms
    pub voice_1: WaveTableOscillatorConfig,
    /// Envelope settings for voice 1
    pub voice_1_env: EnvelopConfiguration,

    /// The waveform for voice two, this is one of the available WaveTable waveforms
    pub voice_2: WaveTableOscillatorConfig,
    /// Envelope settings for voice 2
    pub voice_2_env: EnvelopConfiguration,

    /// The waveform for voice two, this is one of the available WaveTable waveforms
    pub voice_3: WaveTableOscillatorConfig,
    /// Envelope settings for voice 2
    pub voice_3_env: EnvelopConfiguration,

    /// Filter configuration, including cutoff and resonance
    pub filter_config: FilterConfig,
    pub mixer_config: MixerConfiguration,
    pub overdrive_config: OverdriveConfiguration,    
}

impl Patch {
    pub fn default() -> Patch {
        let patch: Patch = Patches::get_patch(Patches::BassGuitar);
        patch
    }
}