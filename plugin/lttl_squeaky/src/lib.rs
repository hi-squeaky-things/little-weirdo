use nih_plug::prelude::*;
use std::{f32::consts, u16};
use std::sync::Arc;
use little_weirdo::synth::{patches::{Patches}};
use little_weirdo::synth::effects::filter::FilterConfig;
use little_weirdo::synth::wavetable_oscillator::Waveform;
use little_weirdo::synth::patch::Patch;
use little_weirdo::synth::{self, Synth};

/// A test tone generator that can either generate a sine wave based on the plugin's parameters or
/// based on the current MIDI input.
pub struct LttLSqueaky {
    params: Arc<LttLSqueakyParams>,
    synth: Synth,
    current_patch: Patches,
}

#[derive(Params)]
struct LttLSqueakyParams {
    #[id = "glide"]
    pub glide: BoolParam,

    /// The type of broken pitch shifting to apply.
    #[id = "patch"]
    pub patch: EnumParam<PatchChooser>,
}
   
#[derive(Enum, Debug, PartialEq)]
enum PatchChooser {
       #[id = "bass_guitar"]
       #[name = "Bass Guitar"]
       bass_guitar
       ,
       #[id = "eletric_piano"]
       #[name = "Eletric Piano"]
       eletric_piano,
   }

impl Default for LttLSqueaky {
    fn default() -> Self {

        let current_patch = Patches::BassGuitar;
        let patch = Patches::get_patch(current_patch);
        let synth = synth::Synth::new(44100, patch);


        Self {
            params: Arc::new(LttLSqueakyParams::default()),
            current_patch: Patches::BassGuitar,
            synth: synth,
        }
    }
}

impl Default for LttLSqueakyParams {
    fn default() -> Self {
        Self {
            glide: BoolParam::new("Gliding", false),
            patch: EnumParam::new("Patch", PatchChooser::bass_guitar),
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
        match self.params.patch.value() {
            PatchChooser::bass_guitar => {
                if self.current_patch != Patches::BassGuitar {
                    let current_patch = Patches::BassGuitar;
                    let patch = Patches::get_patch(current_patch);
                    self.current_patch = current_patch;
                    self.synth.load_patch(patch);
                }
            },
            PatchChooser::eletric_piano => {
                if self.current_patch != Patches::ElectricPiano {
                    let current_patch = Patches::ElectricPiano;
                    let patch = Patches::get_patch(current_patch);
                    self.current_patch = current_patch;
                    self.synth.load_patch(patch);
                }
            }
        }
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
            let synth_sample_after_clock = self.synth.clock_and_output() as f32 / u16::MAX as f32;
            for sample in channel_samples {
                *sample = synth_sample_after_clock;
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