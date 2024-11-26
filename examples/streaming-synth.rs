use console::Key;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample,
};
use cpal::{Device, Host, Sample, Stream, StreamConfig};
use little_weirdo::synth::envelope::Envelop;
use little_weirdo::synth::filter::FilterConfig;
use little_weirdo::synth::math::percentage;
use little_weirdo::synth::oscillator::Waveform;
use little_weirdo::synth::patch::Patch;
use little_weirdo::synth::{self, Synth};
use midi_control::{self, MidiMessage};
use midir;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread::{self};
use std::time::Duration;

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

    // TODO: osc for lfo need beter calculation because with 17Hz is need 2823 wavetable lookup entries.
    let patch = Patch {
        voice_1: Waveform::Bass,
        voice_1_mix_level: 80,
        voice_2: Waveform::Bass,
        voice_2_mix_level: 80,
        lfo_1: 17,
        lfo_1_mix_level: 0,
        voice_1_env: Envelop {
            attack_time: 10,
            decay_time: 150,
            release_time: 150,
            sustain_level: 50,
        },
        voice_2_env: Envelop {
            attack_time: 10,
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
        glide_rate: 5,
        mono: false,
    };
    let mut synth: synth::Synth = synth::Synth::new(44100, patch);

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
                    let synth_sample_after_clock = Sample::from_sample(synth.clock_and_output());
                    for sample in frame.iter_mut() {
                        *sample = synth_sample_after_clock;
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
        MidiMessage::NoteOn(ch, e) => synth.note_on(0, e.key, e.value),
        MidiMessage::NoteOff(ch, e) => synth.note_off(0, e.key),
        MidiMessage::ControlChange(ch, e) => {
            if e.control == 70 {
                //synth.filter.change_freq(e.value as i16 * 10);
            };
        }
        _ => {}
    }
}

fn play_note(synth: &mut synth::Synth, key: Key) {
    if key.eq(&Key::Char('c')) {
        synth.note_on(0, 48, 100);
    }
    if key.eq(&Key::Char('d')) {
        synth.note_off(0, 48);
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
