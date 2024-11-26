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
use rand::rngs::SmallRng;
use rand::RngCore;
use rand::{Rng, SeedableRng};
use rtp_rs::RtpReader;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self};
use std::time::Duration;
use std::{net::UdpSocket, sync::mpsc};
use std::{
    sync::mpsc::channel,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() {
    let stdin_channel: Receiver<Key> = spawn_stdin_channel();
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let own_SSRC = SmallRng::seed_from_u64(0x238293809238u64).next_u32();
    println!("OWN SSRC = {:#01x}", own_SSRC);
    thread::spawn(move || control_loop(own_SSRC));

    let (midi_tx, midi_rx) = mpsc::channel::<MidiMessage>();

    thread::spawn(move || midi_loop(own_SSRC, midi_tx));

    // TODO: osc for lfo need beter calculation because with 17Hz is need 2823 wavetable lookup entries.
    let patch = Patch {
        voice_1: Waveform::Piano,
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
        glide: true,
        glide_rate: 5,
        mono: true,
    };
    let mut synth: synth::Synth = synth::Synth::new(44100, patch);
    let (device, config) = setup_device();
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
                synth.filter.change_freq(e.value as i16 * 10);
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

fn midi_loop(own_SSRC: u32, midi_tx: Sender<MidiMessage>) {
    let midi_port_socket = UdpSocket::bind("[::]:5112").unwrap();
    let mut receive_buf = [0; 2048];
    let mut send_buf = [0; 2048];
    println!("AppleMIDI MIDI loop");
    loop {
        let (amt, src) = midi_port_socket.recv_from(&mut receive_buf).unwrap();
        if let Ok(midipacket) = AppleMIDIReader::new(&mut receive_buf[..amt]) {
            let initiator_TOKEN = midipacket.initiator_token();
            match midipacket.command() {
                AppleMIDICommand::IN => {
                    let len = midipacket.createPacket(
                        AppleMIDICommand::OK,
                        own_SSRC,
                        initiator_TOKEN,
                        &mut send_buf,
                        0,
                    );
                    midi_port_socket.send_to(&mut send_buf[0..len], &src);
                }
                AppleMIDICommand::CK => {
                    println!(
                        "MIDILOOP :: CK command received! count = {}, timestamp = {}, src = {:#01x}",
                        midipacket.amount_of_timestamps(),
                        midipacket.get_timestamp(0),
                        midipacket.SSRC(),
                    );

                    if midipacket.amount_of_timestamps() == 1 {
                        let len = midipacket.createPacket(
                            AppleMIDICommand::CK,
                            own_SSRC,
                            initiator_TOKEN,
                            &mut send_buf,
                            midipacket.get_timestamp(0),
                        );
                        println!("S = {:?}", &send_buf[0..len]);

                        midi_port_socket.send_to(&mut send_buf[0..len], &src);
                    }
                }
                AppleMIDICommand::BY => {
                    println!("MIDILOOP :: BY command received!");
                }
                _ => {
                    println!("MIDILOOP :: Unknown");
                }
            }
        } else {
            if let Ok(rtp) = RtpReader::new(&mut receive_buf[..amt]) {
                //          println!("Sequence number {:?}", rtp.sequence_number());
                //            println!("Payload length {:?}", rtp.payload().len());
                //              println!("Payload type {:#01x}", rtp.payload_type());
                if rtp.payload_type() == 0x61u8 {
                    //                    println!("parse MIDI PACKET!");
                    let message = parse_midi_payload(rtp.payload());
                    println!("message = {:?}", message);
                    midi_tx.send(message).unwrap();

                    //   if (rtp.payload()[0] >> 0 & 1) {
                    //      let midi_message_length: u16 =
                    //  }
                    //[67, 144, 60, 100, 32, 36, 216, 0, 6, 8, 0, 119, 8] note-on C3
                    //[67, 128, 60, 64, 32, 36, 216, 0, 7, 8, 129, 241, 60, 100] note-off C3
                    //                    parse_midi_payload(rtp.payload());
                }
                //
            } else {
                println!("UNKNOWN PACKET?");
            }
        }
    }
}

fn parse_midi_payload(payload: &[u8]) -> MidiMessage {
    let b: bool = payload[0] & 0b1000_0000 != 0;
    let len = payload[0] & 0b0000_1111;
    println!("length {}", len);
    println!("b {}", b);

    if b {}

    MidiMessage::from(&payload[1..4])
}

fn control_loop(own_SSRC: u32) {
    let control_port_socket = UdpSocket::bind("[::]:5111").unwrap();

    let mut receive_buf = [0; 2048];
    let mut send_buf = [0; 2048];
    println!("AppleMIDI control loop");
    loop {
        let (amt, src) = control_port_socket.recv_from(&mut receive_buf).unwrap();
        //      if let Ok(rtp) = RtpReader::new(&mut receive_buf[..amt]) {
        //         println!("Sequence number {:?}", rtp.sequence_number());
        //       println!("Payload length {:?}", rtp.payload().len());
        //  }
        if let Ok(midipacket) = AppleMIDIReader::new(&mut receive_buf[..amt]) {
            //     println!("version = {}", midipacket.protocol_version());
            //    println!("initiator token = {:#01x}", midipacket.initiator_token());
            println!("SSRC = {:#01x}", midipacket.SSRC());
            let initiator_TOKEN = midipacket.initiator_token();
            match midipacket.command() {
                AppleMIDICommand::IN => {
                    //         println!("IN command received!");
                    let len = midipacket.createPacket(
                        AppleMIDICommand::OK,
                        own_SSRC,
                        initiator_TOKEN,
                        &mut send_buf,
                        0,
                    );
                    control_port_socket.send_to(&mut send_buf[0..len], &src);
                }
                AppleMIDICommand::BY => {
                    println!("BY command received!");
                }
                _ => {
                    println!("Unknown");
                }
            }
        }
    }
}

/// Wrapper around a byte-slice of RTP data, providing accessor methods for the RTP header fields.
pub struct AppleMIDIReader<'a> {
    buf: &'a [u8],
}

/// Reasons for `RtpHeader::new()` to fail
#[derive(Debug)]
pub enum AppleMIDIReaderError {
    /// Buffer too short to be valid RTP packet
    BufferTooShort(usize),
    MagicNumberMismatch,
}

#[derive(Debug)]
pub enum AppleMIDICommand {
    IN,
    BY,
    OK,
    NO,
    CK,
    XX,
}

impl<'a> AppleMIDIReader<'a> {
    const MIN_HEADER_LEN: usize = 4;

    pub fn new(b: &'a [u8]) -> Result<AppleMIDIReader<'_>, AppleMIDIReaderError> {
        if b.len() < Self::MIN_HEADER_LEN {
            return Err(AppleMIDIReaderError::BufferTooShort(b.len()));
        }
        if !(b[0] & 0xff == 0xff && b[1] & 0xff == 0xff) {
            return Err(AppleMIDIReaderError::MagicNumberMismatch);
        }
        let r = AppleMIDIReader { buf: b };
        println!("R = {:?}", r.buf);
        Ok(r)
    }

    fn command(&self) -> AppleMIDICommand {
        let command_received = u16::from_be_bytes([self.buf[2], self.buf[3]]);
        //    println!("{:#01x}", command_received);
        match command_received {
            0x494E => {
                // IN
                AppleMIDICommand::IN
            }
            0x4259 => {
                // BY
                AppleMIDICommand::BY
            }
            0x434B => {
                // CK
                AppleMIDICommand::CK
            }
            _ => AppleMIDICommand::XX,
        }
    }

    fn protocol_version(&self) -> u32 {
        u32::from_be_bytes([self.buf[4], self.buf[5], self.buf[6], self.buf[7]])
    }
    fn initiator_token(&self) -> u32 {
        u32::from_be_bytes([self.buf[8], self.buf[9], self.buf[10], self.buf[11]])
    }
    fn SSRC(&self) -> u32 {
        match self.command() {
            AppleMIDICommand::BY | AppleMIDICommand::IN => {
                u32::from_be_bytes([self.buf[12], self.buf[13], self.buf[14], self.buf[15]])
            }
            AppleMIDICommand::CK => {
                u32::from_be_bytes([self.buf[5], self.buf[6], self.buf[7], self.buf[8]])
            }
            _ => 0,
        }
    }

    fn amount_of_timestamps(&self) -> u8 {
        self.buf[8] + 1
    }

    fn get_timestamp(&self, index: u8) -> u64 {
        u64::from_be_bytes([
            self.buf[16 + index as usize * 8],
            self.buf[16 + index as usize * 8 + 1],
            self.buf[16 + index as usize * 8 + 2],
            self.buf[16 + index as usize * 8 + 3],
            self.buf[16 + index as usize * 8 + 4],
            self.buf[16 + index as usize * 8 + 5],
            self.buf[16 + index as usize * 8 + 6],
            self.buf[16 + index as usize * 8 + 7],
        ])
    }

    fn createPacket(
        &self,
        command: AppleMIDICommand,
        own_SSRC: u32,
        initiator_TOKEN: u32,
        buffer: &mut [u8],
        received_timestamp: u64,
    ) -> usize {
        // magic number
        buffer[0] = 0xFFu8;
        buffer[1] = 0xFFu8;

        match command {
            AppleMIDICommand::OK => {
                // version number (2)
                buffer[4] = 0x00u8;
                buffer[5] = 0x00u8;
                buffer[6] = 0x00u8;
                buffer[7] = 0x02u8;

                // initiator_TOKEN
                let mut bytes = initiator_TOKEN.to_be_bytes();
                buffer[8] = bytes[0];
                buffer[9] = bytes[1];
                buffer[10] = bytes[2];
                buffer[11] = bytes[3];

                // ssrc_TOKEN
                bytes = own_SSRC.to_be_bytes();
                buffer[12] = bytes[0];
                buffer[13] = bytes[1];
                buffer[14] = bytes[2];
                buffer[15] = bytes[3];

                // name
                buffer[16] = 0x4Fu8;
                buffer[17] = 0x4Fu8;
                buffer[18] = 0x4Fu8;
                buffer[19] = 0x4Fu8;
            }
            AppleMIDICommand::CK => {
                // ssrc_TOKEN
                let bytes = own_SSRC.to_be_bytes();
                buffer[4] = bytes[0];
                buffer[5] = bytes[1];
                buffer[6] = bytes[2];
                buffer[7] = bytes[3];
                // count
                buffer[8] = 0x01;
                // unused
                buffer[9] = 0x00;
                buffer[10] = 0x00;
                buffer[11] = 0x00;
                // received timestamp
                let timestamp = received_timestamp.to_be_bytes();
                buffer[12] = timestamp[0];
                buffer[13] = timestamp[1];
                buffer[14] = timestamp[2];
                buffer[15] = timestamp[3];
                buffer[16] = timestamp[4];
                buffer[17] = timestamp[5];
                buffer[18] = timestamp[6];
                buffer[19] = timestamp[7];
                // current time
                let now: [u8; 8] = (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64)
                    .to_be_bytes();
                buffer[20] = now[0];
                buffer[21] = now[1];
                buffer[22] = now[2];
                buffer[23] = now[3];
                buffer[24] = now[4];
                buffer[25] = now[5];
                buffer[26] = now[6];
                buffer[27] = now[7];
                //
                buffer[28] = 0x00u8;
                buffer[29] = 0x00u8;
                buffer[30] = 0x00u8;
                buffer[31] = 0x00u8;
                buffer[32] = 0x00u8;
                buffer[33] = 0x00u8;
                buffer[34] = 0x00u8;
                buffer[35] = 0x00u8;
            }
            _ => {
                buffer[2] = 0x00u8;
                buffer[3] = 0x00u8;
            }
        }

        match command {
            AppleMIDICommand::OK => {
                buffer[2] = 0x4Fu8; //O = 4F
                buffer[3] = 0x4Bu8; //K = 4B
                return 20;
            }
            AppleMIDICommand::CK => {
                buffer[2] = 0x43u8;
                buffer[3] = 0x4Bu8;
                return 36;
            }
            _ => {}
        }
        0
    }
}
