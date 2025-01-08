use console::Key;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}
};
use cpal::{Device, Sample, StreamConfig};
use little_weirdo::synth::{self, data::wavetables::{BoxedWavetable, BoxedWavetables}, effects::{filter::{FilterConfig, KindOfFilter}, overdrive::{KindOfOverdrive, OverdriveConfiguration}}, envelope::EnvelopConfiguration, mixer::MixerConfiguration, patch::{Patch, SynthConfiguration, SynthMode}, router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute}, wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig}};

use little_weirdo::synth::{Synth};
use midi_control::{self, MidiMessage};
use midir;
use std::{mem, rc::Rc, sync::mpsc};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread::{self};

    

fn main() {
    let midi_input = midir::MidiInput::new("MIDITest").unwrap();

    let device_port = find_port(&midi_input);
    if device_port.is_none() {
        println!("Input device not found!");
    }

    let (sender, receiver) = channel::<midi_control::MidiMessage>();

    let device_port = device_port.unwrap();

    let (device, config) = setup_device();
    let stdin_channel: Receiver<Key> = spawn_stdin_channel();
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    
    let mut wt_on_heap = BoxedWavetables::new();
    wt_on_heap.add(BoxedWavetable::new(include_bytes!("wav0.lwt")));
    wt_on_heap.add(BoxedWavetable::new(include_bytes!("wav1.lwt")));
    wt_on_heap.add(BoxedWavetable::new(include_bytes!("wav2.lwt")));
    wt_on_heap.add(BoxedWavetable::new(include_bytes!("wav3.lwt")));
    wt_on_heap.add(BoxedWavetable::new(include_bytes!("wav4.lwt")));
    wt_on_heap.add(BoxedWavetable::new(include_bytes!("wav5.lwt")));
    let wt = Rc::new(wt_on_heap);



   let patch = Patch 
   {
    synth_config: SynthConfiguration {
        mode: SynthMode::Mono,
    },
    voices: [
        WaveTableOscillatorConfig {
            soundbank_index: 0,
            glide: false,
            glide_rate: 200,
            detune: 0,
            freq: 440,
            freq_detune: 0,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 9,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq: 440,
            freq_detune: 1,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 1,
            glide: false,
            glide_rate: 200,
            detune: 0,
            freq: 440,
            freq_detune: 2,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 9,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq: 440,
            freq_detune: 3,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 1,
            glide: true,
            glide_rate: 200,
            detune: 0,
            freq: 440,
            freq_detune: 0,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 9,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq: 440,
            freq_detune: 1,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 1,
            glide: false,
            glide_rate: 200,
            detune: 0,
            freq: 440,
            freq_detune: 2,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 9,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq: 440,
            freq_detune: 3,
        },
    ],
    envelops: [
        EnvelopConfiguration {
            attack_time: 10,
            decay_time: 100,
            release_time: 300,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 200,
            decay_time: 10,
            release_time: 100,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 10,
            decay_time: 100,
            release_time: 300,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 200,
            decay_time: 10,
            release_time: 100,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 10,
            decay_time: 100,
            release_time: 300,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 200,
            decay_time: 10,
            release_time: 100,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 10,
            decay_time: 100,
            release_time: 300,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 200,
            decay_time: 10,
            release_time: 100,
            sustain_level: 90,
        },
    ],
    lfos: [
        WaveTableLoFreqOscillatorConfig {
            soundbank_index: 2,
            time: 200,
        },
        WaveTableLoFreqOscillatorConfig {
            soundbank_index: 2,
            time: 200,
        },
        WaveTableLoFreqOscillatorConfig {
            soundbank_index: 2,
            time: 200,
        },
        WaveTableLoFreqOscillatorConfig {
            soundbank_index: 2,
            time: 200,
        },
    ],
    routering_config: RoutingConfiguration {
        voices_to_envelop: [
            VoiceToEnvelopRoute { env: 0 },
            VoiceToEnvelopRoute { env: 1 },
            VoiceToEnvelopRoute { env: 2 },
            VoiceToEnvelopRoute { env: 3 },
            VoiceToEnvelopRoute { env: 4 },
            VoiceToEnvelopRoute { env: 5 },
            VoiceToEnvelopRoute { env: 6 },
            VoiceToEnvelopRoute { env: 7 },
        ],
        voice_to_lfo: [
            VoiceToLFORoute {
                enable: false,
                voices: [1,255],
            },
            VoiceToLFORoute {
                enable: false,
                voices: [1,255],
            },
            VoiceToLFORoute {
                enable: false,
                voices: [1,255],
            },
            VoiceToLFORoute {
                enable: false,
                voices: [1,255],
            },
        ],
        lfo_to_filter: false,
    },
    filter_config: FilterConfig {
        cutoff_frequency: 1_000,
        resonance: 6_000,
        enabled: false,
        kind_of_filter: KindOfFilter::High,
    },
    mixer_config: MixerConfiguration {
        gain_voices: [40, 0, 0, 0, 0, 0, 0, 0],
        gain_main: 50,
    },
    overdrive_config: OverdriveConfiguration {
        threshold: 1000,
        kind: KindOfOverdrive::Softer,
        enabled: false,
    },
    };

    let mut synth: synth::Synth = synth::Synth::new(44100, &patch, Rc::clone(&wt));

    let (midi_tx, midi_rx) = mpsc::channel::<midi_control::MidiMessage>();

    let _connect_in = midi_input.connect(
        &device_port,
        "IAC Driver",
        move |_timestamp, data, sender| {
            let msg: midi_control::MidiMessage = midi_control::MidiMessage::from(data);
            midi_tx.send(msg);
        },
        sender,
    );

    

    //
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                match midi_rx.try_recv() {
                    Ok(message) => process_midimessage(&mut synth, message),
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
                }
                match stdin_channel.try_recv() {
                    Ok(key) => play_note(&mut synth, key),
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
                }
                for frame in data.chunks_mut(2) {
                    let output = synth.clock_and_output();
                    let left:f32 = Sample::from_sample(output[0]);
                    let right:f32 = Sample::from_sample(output[1]);
                
                    let mut count = 0;
                    for sample in frame.iter_mut() {
                        if count == 0 {
                            *sample = left;
                        } else {
                            *sample = right;

                        }
                        count = count + 1;
                    }
                }
            },
            err_fn,
            None,
        )
        .unwrap();
    stream.play().unwrap();
    loop {}
}

fn setup_device() -> (Device, StreamConfig) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))
        .unwrap();
    println!("Output device : {}", device.name().unwrap());
    let supported_config: cpal::SupportedStreamConfig = device.default_output_config().unwrap();
    let config = supported_config.into();
    println!("Config : {:?}", config);
    return (device, config);
}

fn process_midimessage(synth: &mut synth::Synth, command: MidiMessage) {
    match command {
        MidiMessage::NoteOn(ch, e) => synth.note_on(e.key, e.value),
        MidiMessage::NoteOff(ch, e) => synth.note_off( e.key),
        _ => {}
    }
}

fn play_note(synth: &mut synth::Synth, key: Key) {
    if key.eq(&Key::Char('x')) {
        synth.note_on( 40, 100);
    }
    if key.eq(&Key::Char('s')) {
        synth.note_off( 40);
    }
    if key.eq(&Key::Char('c')) {
        synth.note_on( 60, 100);
    }
    if key.eq(&Key::Char('d')) {
        synth.note_off( 60);
    }
}

fn spawn_stdin_channel() -> Receiver<Key> {
    let (tx, rx) = mpsc::channel::<Key>();
    let term = console::Term::stdout();
    thread::spawn(move || loop {
        let key = term.read_key().unwrap();
        tx.send(key).unwrap();
    });
    rx
}

fn find_port<T>(midi_io: &T) -> Option<T::Port>
where
    T: midir::MidiIO,
{
    let mut device_port: Option<T::Port> = None;
    for port in midi_io.ports() {
        if let Ok(port_name) = midi_io.port_name(&port) {
            if port_name.contains("IAC Driver") {
                device_port = Some(port);
                break;
            }
        }
    }
    device_port
}
