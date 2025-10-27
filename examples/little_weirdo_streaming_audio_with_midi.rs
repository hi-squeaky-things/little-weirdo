use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, StreamConfig};

use little_weirdo::synth::{
    self,
    data::wavetables::{BoxedWavetable, BoxedWavetables},
};
use midi_control::{self, MidiMessage};
use midir;
use std::sync::mpsc::channel;
use std::sync::mpsc::TryRecvError;
use std::{
    fs,
    sync::{mpsc, Arc},
};

fn main() {
    // Initialize MIDI input with a client name
    let midi_input = midir::MidiInput::new("MIDITest").unwrap();

    // Find the MIDI input port (IAC Driver)
    let device_port = find_port(&midi_input);
    if device_port.is_none() {
        println!("Input device not found!");
    }

    // Create a channel for MIDI messages (though receiver isn't used in this scope)
    let (sender, _receiver) = channel::<midi_control::MidiMessage>();

    // Unwrap the device port (we know it exists from the check above)
    let device_port = device_port.unwrap();

    // Set up audio output device and configuration
    let (device, config) = setup_device();


    // Define error callback for audio stream
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    // Create a collection of wavetables and load them from files
    let mut wt_on_heap = BoxedWavetables::new();
    for id in 0..10 {
        let filename = format!("examples/soundbank/soundbank_pure_elektro/src/wav{}.raw", id);
        let contents = fs::read(filename).unwrap();
        let bytes: &[u8] = &contents;
        wt_on_heap.add(BoxedWavetable::new(bytes));
    }
    // Wrap wavetables in an Arc for thread-safe sharing
    let wt = Arc::new(wt_on_heap);

    // Load a synth patch from a JSON file
    let patch = serde_json::from_slice(include_bytes!("patches/raw_square.json")).unwrap();

    // Initialize the synthesizer with sample rate, patch, and wavetables
    let mut synth: synth::Synth = synth::Synth::new(44100, &patch, Arc::clone(&wt));

    // Create a channel specifically for MIDI messages from the input device
    let (midi_tx, midi_rx) = mpsc::channel::<midi_control::MidiMessage>();

    // Connect to the MIDI input port and start listening for messages
    let _connect_in = midi_input.connect(
        &device_port,
        "IAC Driver",
        move |_timestamp, data, _sender| {
            // Convert raw MIDI data to our MidiMessage type
            let msg: midi_control::MidiMessage = midi_control::MidiMessage::from(data);
            // Send the message through the channel
            let _ = midi_tx.send(msg);
        },
        sender, // This sender isn't actually used in the callback
    );

    // Build and start the audio output stream
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Process any pending MIDI messages
                match midi_rx.try_recv() {
                    Ok(message) => process_midimessage(&mut synth, message),
                    Err(TryRecvError::Empty) => {} // No messages available
                    Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
                }


                // Generate audio output by processing the synth in chunks
                for frame in data.chunks_mut(2) {
                    // Get the next sample from the synth
                    let output = synth.clock_and_output();

                    // Convert samples to f32 format
                    let left: f32 = Sample::from_sample(output[0]);
                    let right: f32 = Sample::from_sample(output[1]);

                    // Write stereo output to the audio buffer
                    let mut count = 0;
                    for sample in frame.iter_mut() {
                        if count == 0 {
                            *sample = left; // Left channel
                        } else {
                            *sample = right; // Right channel
                        }
                        count = count + 1;
                    }
                }
            },
            err_fn, // Error callback
            None,   // No specific stream configuration
        )
        .unwrap();

    // Start audio playback
    stream.play().unwrap();

    // Main loop - keeps the program running
    loop {}
}

/// Sets up the audio output device and configuration
fn setup_device() -> (Device, StreamConfig) {
    // Get the default audio host
    let host = cpal::default_host();
    // Get the default output device
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))
        .unwrap();

    println!("Output device : {}", device.name().unwrap());

    // Get the default output configuration
    let supported_config: cpal::SupportedStreamConfig = device.default_output_config().unwrap();
    let config = supported_config.into();

    println!("Config : {:?}", config);
    (device, config)
}

/// Processes MIDI messages and updates the synthesizer state
fn process_midimessage(synth: &mut synth::Synth, command: MidiMessage) {
    match command {
        // Handle note-on messages
        MidiMessage::NoteOn(_ch, e) => synth.note_on(e.key, e.value),
        // Handle note-off messages
        MidiMessage::NoteOff(_ch, e) => synth.note_off(e.key),
        // Ignore other message types
        _ => {}
    }
}





/// Finds a MIDI port by name (looking for "IAC Driver")
fn find_port<T>(midi_io: &T) -> Option<T::Port>
where
    T: midir::MidiIO,
{
    let mut device_port: Option<T::Port> = None;

    // Iterate through available ports
    for port in midi_io.ports() {
        if let Ok(port_name) = midi_io.port_name(&port) {
            // Check if port name contains "IAC Driver"
            if port_name.contains("IAC Driver") {
                device_port = Some(port);
                break;
            }
        }
    }
    device_port
}
