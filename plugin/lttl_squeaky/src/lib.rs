use little_weirdo::synth::Synth;
use little_weirdo_soundbanks::soundbanks::SOUND_BANK_PURE_ELEKTRO;
use nih_plug::prelude::*;
use std::{fs, u16};
use std::sync::Arc;
use little_weirdo::synth::{self, data::wavetables::{BoxedWavetable, BoxedWavetables}, effects::{filter::{FilterConfig, KindOfFilter}, overdrive::{KindOfOverdrive, OverdriveConfiguration}}, envelope::EnvelopConfiguration, mixer::MixerConfiguration, patch::{Patch, SynthConfiguration, SynthMode}, router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute}, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig}};


/// A test tone generator that can either generate a sine wave based on the plugin's parameters or
/// based on the current MIDI input.
pub struct LttLSqueaky {
    params: Arc<LttLSqueakyParams>,
    synth: Synth,
}

#[derive(Params)]
struct LttLSqueakyParams {
    #[id = "glide"]
    pub glide: BoolParam,

}
   

impl Default for LttLSqueaky {
    fn default() -> Self {

        let mut wt_on_heap = BoxedWavetables::new();
        for id in 0..10 {
            let filename = format!("examples/wav{}.lwt", id);
            let contents = fs::read(filename).unwrap();
            let bytes: &[u8] = &contents;
            wt_on_heap.add(BoxedWavetable::new(bytes));
       
        }
        let wt = Arc::new(wt_on_heap);

        

       
    let patch = Patch {
        synth_config: SynthConfiguration {
            mode: SynthMode::Mono,
        },
        voices: [
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 8,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 4,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
        ],
        envelops: [
            EnvelopConfiguration {
                attack_time: 200,
                decay_time: 100,
                release_time: 100,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 200,
                decay_time: 100,
                release_time: 100,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 10,
                release_time: 10,
                sustain_level: 100,
            },
        ],
        lfos: [
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 0,
                time: 10,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 4,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 4,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 4,
                time: 200,
            },
        ],
        routering_config: RoutingConfiguration {
            voices_to_envelop: [
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 2 },
                VoiceToEnvelopRoute { env: 3 },
                VoiceToEnvelopRoute { env: 4 },
                VoiceToEnvelopRoute { env: 5 },
                VoiceToEnvelopRoute { env: 6 },
                VoiceToEnvelopRoute { env: 7 },
            ],
            voice_to_lfo: [
                VoiceToLFORoute {
                    enable: true,
                    voices: [0, 1],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
            ],
            lfo_to_filter: false,
        },
        filter_config: FilterConfig {
            cutoff_frequency: 1_000,
            resonance: 6_000,
            enabled: false,
            kind_of_filter: KindOfFilter::Low,
        },
        mixer_config: MixerConfiguration {
            gain_voices: [30, 30, 0, 0, 0, 0, 0, 0],
            gain_main: 80,
        },
        overdrive_config: OverdriveConfiguration {
            threshold: 2000,
            kind: KindOfOverdrive::Softer,
            enabled: false,
        },
    };


    let mut synth: synth::Synth = synth::Synth::new(44100, &patch, Arc::clone(&wt));



        Self {
            params: Arc::new(LttLSqueakyParams::default()),
            synth: synth,
        }
    }
}

impl Default for LttLSqueakyParams {
    fn default() -> Self {
        Self {
            glide: BoolParam::new("Gliding", false),
        }
    }
}



impl Plugin for LttLSqueaky {
    const NAME: &'static str = "LttLSqueaky";
    const VENDOR: &'static str = "HiSqueakyThings";
    const URL: &'static str = "x";
    const EMAIL: &'static str = "daniel@HiSqueakyThings.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            // This is also the default and can be omitted here
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {
     
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut next_event = context.next_event();
       for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                    if event.timing() > sample_id as u32 {
                        break;
                    }

                    match event {
                        NoteEvent::NoteOn { note, velocity, .. } => {
                            self.synth.note_on( note, 100)
                        }
                        NoteEvent::NoteOff { note, .. }  => {
                            self.synth.note_off( note)
                        }
                        NoteEvent::PolyPressure { note, pressure, .. } =>
                        {
                        }
                        _ => (),
                    }

                    next_event = context.next_event();
               

            }
            let output = self.synth.clock_and_output();
            let left = output[0] as f32 / u16::MAX as f32;
            let right = output[1] as f32 / u16::MAX as f32;
            let mut count = 0;
            for sample in channel_samples {
                if count == 0 {
                    *sample = left;
                } else {
                    *sample = right;

                }
                count = count + 1;
            }
        }

        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for LttLSqueaky {
    const CLAP_ID: &'static str = "com.hisqueakythings.lttlsqueaky";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("An optionally MIDI controlled sine test tone");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for LttLSqueaky {
    const VST3_CLASS_ID: [u8; 16] = *b"[[LttLSqueakY-2]";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
        Vst3SubCategory::Tools,
    ];
}

nih_export_clap!(LttLSqueaky);
nih_export_vst3!(LttLSqueaky);