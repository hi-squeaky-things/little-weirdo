use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, Stream, StreamConfig};
use crossterm::{
    event::{
        self, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use little_weirdo::synth::data::patches::{BoxedPatch, BoxedPatches};
use little_weirdo::synth::{
    self,
    data::waveforms::{BoxedWaveform, BoxedWaveforms, Waveforms},
};
use midi_control::MidiMessage;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph, Tabs},
    Terminal,
};
use std::{
    collections::HashSet,
    fs,
    io::{self, Stdout},
    sync::{
        atomic::{AtomicU32, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    time::Duration,
};

const LIVE_SAMPLE_COUNT: usize = 256;

struct LiveAudio {
    samples: [AtomicU32; LIVE_SAMPLE_COUNT],
    write_index: AtomicUsize,
}

impl LiveAudio {
    fn new() -> Self {
        Self {
            samples: std::array::from_fn(|_| AtomicU32::new(0.0f32.to_bits())),
            write_index: AtomicUsize::new(0),
        }
    }

    fn push(&self, sample: f32) {
        let index = self.write_index.load(Ordering::Relaxed);
        self.samples[index % LIVE_SAMPLE_COUNT].store(sample.to_bits(), Ordering::Relaxed);
        self.write_index
            .store(index.wrapping_add(1), Ordering::Release);
    }

    fn snapshot(&self) -> Vec<(f64, f64)> {
        let end = self.write_index.load(Ordering::Acquire);
        let count = end.min(LIVE_SAMPLE_COUNT);
        let mut snapshot = vec![(0.0, 0.0); LIVE_SAMPLE_COUNT];
        for offset in 0..count {
            let index = end.wrapping_sub(count).wrapping_add(offset);
            let sample =
                f32::from_bits(self.samples[index % LIVE_SAMPLE_COUNT].load(Ordering::Relaxed));
            let x = (LIVE_SAMPLE_COUNT - count + offset) as f64;
            snapshot[LIVE_SAMPLE_COUNT - count + offset] = (x, sample as f64);
        }
        snapshot
    }
}

#[derive(Clone, Copy)]
enum FmParameter {
    CarrierWaveform,
    ModulatorWaveform,
    ModulatorRatio,
    ModulationDepth,
}

#[derive(Clone, Copy)]
enum EffectParameter {
    DelayTime,
    DelayMix,
    DelayFeedback,
    FlangerBaseDelay,
    FlangerDepth,
    FlangerRate,
    FlangerMix,
    FilterCutoff,
    FilterResonance,
    FilterKind,
    DistortionKind,
    DistortionThreshold,
}

#[derive(Clone, Copy)]
enum EffectToggle {
    Delay,
    Flanger,
    Filter,
    Distortion,
}

#[derive(Clone, Copy)]
enum EnvelopeParameter {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Menu {
    Fm,
    Voices,
    Effects,
    Envelope,
    Live,
    Devices,
    SavePatch,
}

enum AudioCommand {
    Midi(MidiMessage),
    NoteOn(u8, u8),
    NoteOff(u8),
    AdjustFm(FmParameter, bool),
    AdjustEffect(EffectParameter, bool),
    ToggleEffect(EffectToggle),
    SetEnvelope(EnvelopeParameter, u16),
    ToggleFm(bool),
    ChangeVoice(u8),
}

enum DeviceCommand {
    SwitchAudioDevice(usize),
    SwitchMidiPort(usize),
}

struct SynthPanel {
    fm_enabled: bool,
    carrier_waveform: u8,
    modulator_waveform: u8,
    modulator_ratio: u8,
    modulation_depth_hz: u16,
    selected: usize,
    menu: Menu,
    waveform_count: u8,
    waveform_plots: Vec<Vec<(f64, f64)>>,
    live_audio: Arc<LiveAudio>,
    midi_status: String,
    delay_enabled: bool,
    delay_time_ms: u16,
    delay_mix: u8,
    delay_feedback: u8,
    flanger_enabled: bool,
    flanger_base_delay_ms: u16,
    flanger_depth_ms: u16,
    flanger_rate_hz: u16,
    flanger_mix: u8,
    filter_enabled: bool,
    filter_cutoff: u16,
    filter_resonance: u16,
    filter_kind: u8,
    distortion_enabled: bool,
    distortion_kind: u8,
    distortion_threshold: u16,
    attack_time_ms: u16,
    decay_time_ms: u16,
    sustain_level: u8,
    release_time_ms: u16,
    keyboard_octave_shift: i8,
    held_keyboard_notes: HashSet<u8>,
    audio_devices: Vec<String>,
    selected_audio_device: usize,
    midi_ports: Vec<String>,
    selected_midi_port: usize,
    device_status: String,
    selected_voice: u8,
    patch_name: String,
    save_message: String,
}

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let output_devices = host
        .output_devices()?
        .map(|device| Ok((device.description()?.name().to_owned(), device)))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let audio_devices: Vec<String> = output_devices
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    if output_devices.is_empty() {
        anyhow::bail!("No audio output devices are available");
    }
    let default_output_name = host.default_output_device().and_then(|device| {
        device
            .description()
            .ok()
            .map(|description| description.name().to_owned())
    });
    let selected_audio_device = audio_devices
        .iter()
        .position(|name| Some(name) == default_output_name.as_ref())
        .unwrap_or(0);
    let initial_audio_name = audio_devices[selected_audio_device].clone();
    let device = output_devices[selected_audio_device].1.clone();
    let config: StreamConfig = device.default_output_config()?.into();

    let waveform_count = 55u8;
    let mut waveforms = BoxedWaveforms::new();
    for id in 0..waveform_count {
        let filename = format!(
            "examples/soundbank/synth/waveforms/src/{:03}_sample.raw",
            id
        );
        let contents = fs::read(filename)?;
        waveforms.add(BoxedWaveform::new(&contents));
    }
    let waveforms = Arc::new(waveforms);
    let waveform_plots = (0..waveform_count)
        .map(|index| {
            waveforms
                .get_waveform_reference(index)
                .iter()
                .enumerate()
                .map(|(sample_index, sample)| {
                    (sample_index as f64, *sample as f64 / i16::MAX as f64)
                })
                .collect()
        })
        .collect();

    let patch: synth::patch::Patch = serde_json::from_slice(include_bytes!(
        "../soundbank/synth/patches/fm_basic.json"
    ))?;
    let fm = patch.fm_config;
    let envelope = patch.envelops[0];
    let patch = BoxedPatch::new(patch);
    let mut patches = BoxedPatches::new();
    patches.add(patch);
    let patches = Arc::new(patches);

    let live_audio = Arc::new(LiveAudio::new());

    let (audio_tx, audio_rx) = mpsc::channel::<AudioCommand>();
    let audio_rx = Arc::new(Mutex::new(audio_rx));
    let (device_tx, device_rx) = mpsc::channel::<DeviceCommand>();
    let (status_tx, status_rx) = mpsc::channel::<String>();
    let midi_ports = midi_input_port_names();
    let selected_midi_port = midi_ports
        .iter()
        .position(|name| name.contains("IAC"))
        .unwrap_or(0);
    let (midi_status, midi_connection) = connect_midi_port(
        (!midi_ports.is_empty()).then_some(selected_midi_port),
        audio_tx.clone(),
    );
    let synth = Arc::new(Mutex::new(synth::Synth::new(
        config.sample_rate.min(u16::MAX as u32) as u16,
        0,
        Arc::clone(&patches),
        Arc::clone(&waveforms),
    )));

    // Query initial effect states directly from the synth
    let (delay_enabled, delay_time_ms, delay_mix, delay_feedback) = {
        let mut s = synth.lock().unwrap();
        s.control().delay_settings()
    };
    let (flanger_enabled, flanger_base_delay_ms, flanger_depth_ms, flanger_rate_hz, flanger_mix) = {
        let mut s = synth.lock().unwrap();
        s.control().flanger_settings()
    };
    let (filter_enabled, filter_cutoff, filter_resonance, filter_kind) = {
        let mut s = synth.lock().unwrap();
        s.control().filter_settings()
    };
    let (distortion_enabled, distortion_kind, distortion_threshold) = {
        let mut s = synth.lock().unwrap();
        s.control().overdrive_settings()
    };

    let stream = build_audio_stream(
        &device,
        &config,
        Arc::clone(&synth),
        Arc::clone(&live_audio),
        Arc::clone(&audio_rx),
    )?;
    let midi_audio_tx = audio_tx.clone();
    let manager_live_audio = Arc::clone(&live_audio);
    let audio_thread = std::thread::spawn(move || {
        let mut stream = stream;
        let mut midi_connection = midi_connection;

        while let Ok(command) = device_rx.recv() {
            match command {
                DeviceCommand::SwitchAudioDevice(index) => {
                    let Some((name, device)) = output_devices.get(index) else {
                        let _ = status_tx.send("Audio switch failed: device is unavailable".into());
                        continue;
                    };
                    let new_stream = device
                        .default_output_config()
                        .map_err(anyhow::Error::from)
                        .and_then(|config| {
                            let config: StreamConfig = config.into();
                            build_audio_stream(
                                device,
                                &config,
                                Arc::clone(&synth),
                                Arc::clone(&manager_live_audio),
                                Arc::clone(&audio_rx),
                            )
                        });
                    match new_stream {
                        Ok(new_stream) => {
                            stream = new_stream;
                            let _ = status_tx.send(format!("Audio output: {name}"));
                        }
                        Err(error) => {
                            let _ = status_tx.send(format!("Audio switch failed: {error}"));
                        }
                    }
                }
                DeviceCommand::SwitchMidiPort(index) => {
                    let (status, new_connection) =
                        connect_midi_port(Some(index), midi_audio_tx.clone());
                    if let Some(connection) = new_connection {
                        midi_connection = Some(connection);
                    }
                    let _ = status_tx.send(status);
                }
            }
        }
        drop(stream);
        drop(midi_connection);
    });

    let mut panel = SynthPanel {
        fm_enabled: true,
        carrier_waveform: fm.carrier_waveform,
        modulator_waveform: fm.modulator_waveform,
        modulator_ratio: fm.modulator_ratio.max(1),
        modulation_depth_hz: fm.modulation_depth_hz,
        selected: 0,
        menu: Menu::Fm,
        waveform_count,
        waveform_plots,
        live_audio,
        midi_status,
        delay_enabled,
        delay_time_ms,
        delay_mix,
        delay_feedback,
        flanger_enabled,
        flanger_base_delay_ms,
        flanger_depth_ms,
        flanger_rate_hz,
        flanger_mix,
        filter_enabled,
        filter_cutoff,
        filter_resonance,
        filter_kind,
        distortion_enabled,
        distortion_kind,
        distortion_threshold,
        attack_time_ms: envelope.attack_time,
        decay_time_ms: envelope.decay_time,
        sustain_level: envelope.sustain_level.clamp(0, 100) as u8,
        release_time_ms: envelope.release_time,
        keyboard_octave_shift: 0,
        held_keyboard_notes: HashSet::new(),
        audio_devices,
        selected_audio_device,
        midi_ports,
        selected_midi_port,
        device_status: format!("Audio output: {initial_audio_name}"),
        selected_voice: 0,
        patch_name: "new_patch".to_string(),
        save_message: String::new(),
    };
    let ui_result = run_terminal_ui(&mut panel, &audio_tx, &device_tx, &status_rx);
    drop(device_tx);
    audio_thread
        .join()
        .map_err(|_| anyhow::anyhow!("Audio device manager thread panicked"))?;
    ui_result?;
    Ok(())
}

fn run_terminal_ui(
    panel: &mut SynthPanel,
    audio_tx: &mpsc::Sender<AudioCommand>,
    device_tx: &mpsc::Sender<DeviceCommand>,
    status_rx: &mpsc::Receiver<String>,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = event_loop(&mut terminal, panel, audio_tx, device_tx, status_rx);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        PopKeyboardEnhancementFlags,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    panel: &mut SynthPanel,
    audio_tx: &mpsc::Sender<AudioCommand>,
    device_tx: &mpsc::Sender<DeviceCommand>,
    status_rx: &mpsc::Receiver<String>,
) -> anyhow::Result<()> {
    let mut held_notes = HashSet::new();
    loop {
        while let Ok(status) = status_rx.try_recv() {
            panel.device_status = status;
        }
        terminal.draw(|frame| draw_panel(frame, panel))?;
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if let Some(note_index) = computer_key_index(key.code) {
            let note = keyboard_note(note_index, panel.keyboard_octave_shift);
            match key.kind {
                KeyEventKind::Press if held_notes.insert(note) => {
                    audio_tx.send(AudioCommand::NoteOn(note, 100))?;
                    panel.held_keyboard_notes.insert(note);
                }
                KeyEventKind::Release if held_notes.remove(&note) => {
                    audio_tx.send(AudioCommand::NoteOff(note))?;
                    panel.held_keyboard_notes.remove(&note);
                }
                KeyEventKind::Press | KeyEventKind::Repeat | KeyEventKind::Release => {}
            }
            continue;
        }
        if key.kind == KeyEventKind::Release {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break,
            KeyCode::Char('+') | KeyCode::Char('=') => {
                shift_keyboard_octave(panel, &mut held_notes, audio_tx, 1)?;
            }
            KeyCode::Char('-') => {
                shift_keyboard_octave(panel, &mut held_notes, audio_tx, -1)?;
            }
            KeyCode::Tab => {
                panel.menu = match panel.menu {
                    Menu::Fm => Menu::Voices,
                    Menu::Voices => Menu::Effects,
                    Menu::Effects => Menu::Envelope,
                    Menu::Envelope => Menu::Live,
                    Menu::Live => Menu::Devices,
                    Menu::Devices => Menu::SavePatch,
                    Menu::SavePatch => Menu::Fm,
                };
                panel.selected = panel.selected.min(menu_item_count(panel.menu) - 1);
            }
            KeyCode::Up | KeyCode::Char('k') => panel.selected = panel.selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                panel.selected = (panel.selected + 1).min(menu_item_count(panel.menu) - 1)
            }
            KeyCode::Left | KeyCode::Char('h') => {
                adjust_selected(panel, audio_tx, device_tx, false)?
            }
            KeyCode::Right | KeyCode::Char('l') => {
                adjust_selected(panel, audio_tx, device_tx, true)?
            }
            _ => {}
        }
    }
    for note in held_notes {
        audio_tx.send(AudioCommand::NoteOff(note))?;
    }
    panel.held_keyboard_notes.clear();
    Ok(())
}

fn shift_keyboard_octave(
    panel: &mut SynthPanel,
    held_notes: &mut HashSet<u8>,
    audio_tx: &mpsc::Sender<AudioCommand>,
    direction: i8,
) -> anyhow::Result<()> {
    let next_shift = (panel.keyboard_octave_shift + direction).clamp(-2, 4);
    if next_shift == panel.keyboard_octave_shift {
        return Ok(());
    }
    for note in held_notes.drain() {
        audio_tx.send(AudioCommand::NoteOff(note))?;
    }
    panel.held_keyboard_notes.clear();
    panel.keyboard_octave_shift = next_shift;
    Ok(())
}

fn computer_key_index(key: KeyCode) -> Option<u8> {
    match key {
        KeyCode::Char('1') => Some(0),
        KeyCode::Char('2') => Some(1),
        KeyCode::Char('3') => Some(2),
        KeyCode::Char('4') => Some(3),
        KeyCode::Char('5') => Some(4),
        KeyCode::Char('6') => Some(5),
        KeyCode::Char('7') => Some(6),
        KeyCode::Char('8') => Some(7),
        _ => None,
    }
}

fn keyboard_note(index: u8, octave_shift: i8) -> u8 {
    const SCALE_STEPS: [u8; 8] = [0, 2, 4, 5, 7, 9, 11, 12];
    (48i16 + octave_shift as i16 * 12 + SCALE_STEPS[index as usize] as i16) as u8
}

fn adjust_selected(
    panel: &mut SynthPanel,
    audio_tx: &mpsc::Sender<AudioCommand>,
    device_tx: &mpsc::Sender<DeviceCommand>,
    clockwise: bool,
) -> anyhow::Result<()> {
    if panel.menu == Menu::Voices {
        panel.selected_voice = cycle_waveform(panel.selected_voice, panel.waveform_count, clockwise);
        audio_tx.send(AudioCommand::ChangeVoice(panel.selected_voice))?;
        return Ok(());
    }
    if panel.menu == Menu::Live || panel.menu == Menu::Devices {
        if panel.menu == Menu::Devices {
            if panel.selected == 0 {
                let count = panel.audio_devices.len();
                if count > 0 {
                    let next_idx = if clockwise {
                        (panel.selected_audio_device + 1) % count
                    } else {
                        (panel.selected_audio_device + count - 1) % count
                    };
                    panel.selected_audio_device = next_idx;
                    device_tx.send(DeviceCommand::SwitchAudioDevice(next_idx))?;
                }
            } else if panel.selected == 1 {
                let count = panel.midi_ports.len();
                if count > 0 {
                    let next_idx = if clockwise {
                        (panel.selected_midi_port + 1) % count
                    } else {
                        (panel.selected_midi_port + count - 1) % count
                    };
                    panel.selected_midi_port = next_idx;
                    device_tx.send(DeviceCommand::SwitchMidiPort(next_idx))?;
                }
            }
        }
        return Ok(());
    }
    if panel.menu == Menu::Effects {
        return adjust_effect_selected(panel, audio_tx, clockwise);
    }
    if panel.menu == Menu::Envelope {
        return adjust_envelope_selected(panel, audio_tx, clockwise);
    }

    let parameter = match panel.selected {
        0 => {
            panel.carrier_waveform =
                cycle_waveform(panel.carrier_waveform, panel.waveform_count, clockwise);
            FmParameter::CarrierWaveform
        }
        1 => {
            panel.modulator_waveform =
                cycle_waveform(panel.modulator_waveform, panel.waveform_count, clockwise);
            FmParameter::ModulatorWaveform
        }
        2 => {
            panel.modulator_ratio = if clockwise {
                panel.modulator_ratio.saturating_add(1)
            } else {
                panel.modulator_ratio.saturating_sub(1).max(1)
            };
            FmParameter::ModulatorRatio
        }
        3 => {
            panel.modulation_depth_hz = if clockwise {
                panel.modulation_depth_hz.saturating_add(10)
            } else {
                panel.modulation_depth_hz.saturating_sub(10)
            };
            FmParameter::ModulationDepth
        }
        _ => {
            panel.fm_enabled = !panel.fm_enabled;
            audio_tx.send(AudioCommand::ToggleFm(panel.fm_enabled))?;
            return Ok(());
        }
    };
    audio_tx.send(AudioCommand::AdjustFm(parameter, clockwise))?;
    Ok(())
}

fn menu_item_count(menu: Menu) -> usize {
    match menu {
        Menu::Fm => 5,
        Menu::Voices => 1,
        Menu::Effects => 16,
        Menu::Envelope => 4,
        Menu::Live => 1,
        Menu::Devices => 2,
        Menu::SavePatch => 2,
    }
}

fn adjust_envelope_selected(
    panel: &mut SynthPanel,
    audio_tx: &mpsc::Sender<AudioCommand>,
    clockwise: bool,
) -> anyhow::Result<()> {
    let (parameter, value) = match panel.selected {
        0 => {
            panel.attack_time_ms = step_u16(panel.attack_time_ms, clockwise, 10, 0, 5000);
            (EnvelopeParameter::Attack, panel.attack_time_ms)
        }
        1 => {
            panel.decay_time_ms = step_u16(panel.decay_time_ms, clockwise, 10, 0, 5000);
            (EnvelopeParameter::Decay, panel.decay_time_ms)
        }
        2 => {
            panel.sustain_level = step_u8(panel.sustain_level, clockwise, 1, 0, 100);
            (EnvelopeParameter::Sustain, panel.sustain_level as u16)
        }
        _ => {
            panel.release_time_ms = step_u16(panel.release_time_ms, clockwise, 10, 0, 5000);
            (EnvelopeParameter::Release, panel.release_time_ms)
        }
    };
    audio_tx.send(AudioCommand::SetEnvelope(parameter, value))?;
    Ok(())
}

fn adjust_effect_selected(
    panel: &mut SynthPanel,
    audio_tx: &mpsc::Sender<AudioCommand>,
    clockwise: bool,
) -> anyhow::Result<()> {
    let toggle = match panel.selected {
        0 => Some((EffectToggle::Delay, &mut panel.delay_enabled)),
        4 => Some((EffectToggle::Flanger, &mut panel.flanger_enabled)),
        9 => Some((EffectToggle::Filter, &mut panel.filter_enabled)),
        13 => Some((EffectToggle::Distortion, &mut panel.distortion_enabled)),
        _ => None,
    };
    if let Some((toggle, enabled)) = toggle {
        *enabled = !*enabled;
        audio_tx.send(AudioCommand::ToggleEffect(toggle))?;
        return Ok(());
    }

    let parameter = match panel.selected {
        1 => {
            panel.delay_time_ms = step_u16(panel.delay_time_ms, clockwise, 10, 10, 5000);
            EffectParameter::DelayTime
        }
        2 => {
            panel.delay_mix = step_u8(panel.delay_mix, clockwise, 1, 0, 100);
            EffectParameter::DelayMix
        }
        3 => {
            panel.delay_feedback = step_u8(panel.delay_feedback, clockwise, 1, 0, 100);
            EffectParameter::DelayFeedback
        }
        5 => {
            panel.flanger_base_delay_ms =
                step_u16(panel.flanger_base_delay_ms, clockwise, 1, 0, 2000);
            EffectParameter::FlangerBaseDelay
        }
        6 => {
            panel.flanger_depth_ms = step_u16(panel.flanger_depth_ms, clockwise, 1, 0, 2000);
            EffectParameter::FlangerDepth
        }
        7 => {
            panel.flanger_rate_hz = step_u16(panel.flanger_rate_hz, clockwise, 1, 0, 100);
            EffectParameter::FlangerRate
        }
        8 => {
            panel.flanger_mix = step_u8(panel.flanger_mix, clockwise, 1, 0, 100);
            EffectParameter::FlangerMix
        }
        10 => {
            panel.filter_cutoff = step_u16(panel.filter_cutoff, clockwise, 500, 0, u16::MAX);
            EffectParameter::FilterCutoff
        }
        11 => {
            panel.filter_resonance = step_u16(panel.filter_resonance, clockwise, 500, 0, u16::MAX);
            EffectParameter::FilterResonance
        }
        12 => {
            panel.filter_kind = panel
                .filter_kind
                .wrapping_add(if clockwise { 1 } else { 3 })
                % 4;
            EffectParameter::FilterKind
        }
        14 => {
            panel.distortion_kind =
                panel
                    .distortion_kind
                    .wrapping_add(if clockwise { 1 } else { 2 })
                    % 3;
            EffectParameter::DistortionKind
        }
        _ => {
            panel.distortion_threshold = step_u16(
                panel.distortion_threshold,
                clockwise,
                250,
                0,
                i16::MAX as u16,
            );
            EffectParameter::DistortionThreshold
        }
    };
    audio_tx.send(AudioCommand::AdjustEffect(parameter, clockwise))?;
    Ok(())
}

fn step_u16(value: u16, clockwise: bool, amount: u16, minimum: u16, maximum: u16) -> u16 {
    if clockwise {
        value.saturating_add(amount).min(maximum)
    } else {
        value.saturating_sub(amount).max(minimum)
    }
}

fn step_u8(value: u8, clockwise: bool, amount: u8, minimum: u8, maximum: u8) -> u8 {
    if clockwise {
        value.saturating_add(amount).min(maximum)
    } else {
        value.saturating_sub(amount).max(minimum)
    }
}

fn on_off(enabled: bool) -> &'static str {
    if enabled {
        "ON"
    } else {
        "OFF"
    }
}

fn filter_kind_name(kind: u8) -> &'static str {
    ["Low-pass", "High-pass", "Band-pass", "Notch"][kind as usize % 4]
}

fn distortion_kind_name(kind: u8) -> &'static str {
    ["Hard", "Soft", "Softer"][kind as usize % 3]
}

fn cycle_waveform(current: u8, count: u8, clockwise: bool) -> u8 {
    if clockwise {
        current.wrapping_add(1) % count
    } else {
        current.wrapping_add(count).wrapping_sub(1) % count
    }
}

fn draw_panel(frame: &mut ratatui::Frame<'_>, panel: &SynthPanel) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(" ⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⡇⢸⣏⡉⡇⣏⡱⡏⢱⡎⢱"),
        Line::from(" ⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠟⠻⠧⠤⠇⠇⠱⠧⠜⠣⠜ - Hi Squeaky Things can happen at any time!"),
    ])
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, layout[0]);

    let selected_menu = match panel.menu {
        Menu::Fm => 0,
        Menu::Voices => 1,
        Menu::Effects => 2,
        Menu::Envelope => 3,
        Menu::Live => 4,
        Menu::Devices => 5,
        Menu::SavePatch => 6,
    };
    let menus = Tabs::new(["FM", "VOICES", "EFFECTS", "ENVELOPE", "LIVE", "DEVICES", "SAVE"])
        .select(selected_menu)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::DarkGray))
        .divider("  ")
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(menus, layout[1]);

    let keyboard_block = Block::default()
        .title(format!(
            " KEYBOARD C{}-C{}  (+/- octave) ",
            3 + panel.keyboard_octave_shift,
            4 + panel.keyboard_octave_shift
        ))
        .borders(Borders::ALL);
    let keyboard_area = keyboard_block.inner(layout[2]);
    frame.render_widget(keyboard_block, layout[2]);
    let key_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 8); 8])
        .split(keyboard_area);
    for (index, key_area) in key_areas.iter().enumerate() {
        let note = keyboard_note(index as u8, panel.keyboard_octave_shift);
        let active = panel.held_keyboard_notes.contains(&note);
        let key = Paragraph::new(format!("♪ {} ♪", midi_note_name(note)))
            .alignment(Alignment::Center)
            .style(if active {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightMagenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            })
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(key, *key_area);
    }

    let values = match panel.menu {
        Menu::Fm => vec![
            format!("Carrier waveform       {:03}", panel.carrier_waveform),
            format!("Modulator waveform     {:03}", panel.modulator_waveform),
            format!("Modulator ratio        {} : 1", panel.modulator_ratio),
            format!("Modulation depth       {} Hz", panel.modulation_depth_hz),
            format!("FM Synthesis           {}", on_off(panel.fm_enabled)),
        ],
        Menu::Voices => vec![
            format!("Selected Voice         {:03}", panel.selected_voice),
        ],
        Menu::Effects => vec![
            format!("Delay                 {}", on_off(panel.delay_enabled)),
            format!("  Time                {} ms", panel.delay_time_ms),
            format!("  Mix                 {}%", panel.delay_mix),
            format!("  Feedback            {}%", panel.delay_feedback),
            format!("Flanger               {}", on_off(panel.flanger_enabled)),
            format!("  Base delay          {} ms", panel.flanger_base_delay_ms),
            format!("  Depth               {} ms", panel.flanger_depth_ms),
            format!("  Rate                {} Hz", panel.flanger_rate_hz),
            format!("  Mix                 {}%", panel.flanger_mix),
            format!("Filter                {}", on_off(panel.filter_enabled)),
            format!("  Cutoff              {}", panel.filter_cutoff),
            format!("  Resonance           {}", panel.filter_resonance),
            format!(
                "  Type                {}",
                filter_kind_name(panel.filter_kind)
            ),
            format!("Distortion            {}", on_off(panel.distortion_enabled)),
            format!(
                "  Curve               {}",
                distortion_kind_name(panel.distortion_kind)
            ),
            format!("  Threshold           {}", panel.distortion_threshold),
        ],
        Menu::Envelope => vec![
            format!("Attack                {} ms", panel.attack_time_ms),
            format!("Decay                 {} ms", panel.decay_time_ms),
            format!("Sustain               {}%", panel.sustain_level),
            format!("Release               {} ms", panel.release_time_ms),
        ],
        Menu::Live => Vec::new(),
        Menu::Devices => vec![
            format!(
                "Audio Output       {}",
                panel
                    .audio_devices
                    .get(panel.selected_audio_device)
                    .unwrap_or(&"None".to_string())
            ),
            format!(
                "MIDI Input         {}",
                panel
                    .midi_ports
                    .get(panel.selected_midi_port)
                    .unwrap_or(&"None".to_string())
            ),
        ],
        Menu::SavePatch => vec![
            format!("Patch Name         {}", panel.patch_name),
            format!("Status             {}", panel.save_message),
        ],
    };
    let items = values
        .iter()
        .map(|value| ListItem::new(value.as_str()))
        .collect::<Vec<_>>();
    let settings = List::new(items)
        .block(
            Block::default()
                .title(match panel.menu {
                    Menu::Fm => " FM PARAMETERS ",
                    Menu::Voices => " VOICE SELECTION ",
                    Menu::Effects => " EFFECTS ",
                    Menu::Envelope => " ENVELOPE ",
                    Menu::Live => " LIVE AUDIO ",
                    Menu::Devices => " DEVICES ",
                    Menu::SavePatch => " SAVE PATCH ",
                })
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(panel.selected));
    if panel.menu == Menu::Live {
        let live_samples = panel.live_audio.snapshot();
        let peak = live_samples
            .iter()
            .map(|(_, sample)| sample.abs())
            .fold(0.0f64, f64::max);
        let rms = (live_samples
            .iter()
            .map(|(_, sample)| sample * sample)
            .sum::<f64>()
            / LIVE_SAMPLE_COUNT as f64)
            .sqrt();
        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(78), Constraint::Percentage(22)])
            .split(layout[3]);
        let chart = Chart::new(vec![Dataset::default()
            .name("Output")
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::LightCyan))
            .data(&live_samples)])
        .block(
            Block::default()
                .title(" LIVE AUDIO STREAM - RECENT SAMPLES ")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, (LIVE_SAMPLE_COUNT - 1) as f64])
                .labels(vec![Span::raw("OLD"), Span::raw("NOW")]),
        )
        .y_axis(Axis::default().bounds([-1.0, 1.0]).labels(vec![
            Span::raw("-1"),
            Span::raw("0"),
            Span::raw("1"),
        ]));
        frame.render_widget(chart, content[0]);

        let levels = Paragraph::new(format!("Peak\n{peak:.3}\n\nRMS\n{rms:.3}"))
            .alignment(Alignment::Center)
            .block(Block::default().title(" LEVEL ").borders(Borders::ALL));
        frame.render_widget(levels, content[1]);
    } else if panel.menu == Menu::Envelope {
        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(layout[3]);
        let attack = panel.attack_time_ms as f64;
        let decay = panel.decay_time_ms as f64;
        let sustain = panel.sustain_level as f64 / 100.0;
        let release = panel.release_time_ms as f64;
        let sustain_duration = (attack + decay + release).max(100.0);
        let sustain_end = attack + decay + sustain_duration;
        let release_end = sustain_end + release;
        let envelope_points = vec![
            (0.0, 0.0),
            (attack, 1.0),
            (attack + decay, sustain),
            (sustain_end, sustain),
            (release_end, 0.0),
        ];
        let envelope_chart = Chart::new(vec![Dataset::default()
            .name("Amplitude")
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::LightCyan))
            .data(&envelope_points)])
        .block(
            Block::default()
                .title(" ADSR ENVELOPE ")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, release_end.max(1.0)])
                .labels(vec![
                    Span::raw("0 ms"),
                    Span::raw(format!("{release_end:.0} ms")),
                ]),
        )
        .y_axis(Axis::default().bounds([0.0, 1.0]).labels(vec![
            Span::raw("0"),
            Span::raw("0.5"),
            Span::raw("1"),
        ]));
        frame.render_stateful_widget(settings, content[0], &mut state);
        frame.render_widget(envelope_chart, content[1]);
    } else if panel.menu == Menu::Fm && panel.selected < 2 {
        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(layout[3]);
        frame.render_stateful_widget(settings, content[0], &mut state);

        let waveform_index = if panel.selected == 0 {
            panel.carrier_waveform
        } else {
            panel.modulator_waveform
        };
        let waveform_name = if panel.selected == 0 {
            "CARRIER"
        } else {
            "MODULATOR"
        };
        let samples = &panel.waveform_plots[waveform_index as usize];
        let chart = Chart::new(vec![Dataset::default()
            .name(format!("Waveform {:03}", waveform_index))
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::LightCyan))
            .data(samples)])
        .block(
            Block::default()
                .title(format!(" {waveform_name} WAVEFORM "))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, samples.len().saturating_sub(1) as f64])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{}", samples.len() - 1)),
                ]),
        )
        .y_axis(Axis::default().bounds([-1.0, 1.0]).labels(vec![
            Span::raw("-1"),
            Span::raw("0"),
            Span::raw("1"),
        ]));
        frame.render_widget(chart, content[1]);
    } else if panel.menu == Menu::Voices {
        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(layout[3]);
        frame.render_stateful_widget(settings, content[0], &mut state);

        let samples = &panel.waveform_plots[panel.selected_voice as usize];
        let chart = Chart::new(vec![Dataset::default()
            .name(format!("Voice {:03}", panel.selected_voice))
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::LightCyan))
            .data(samples)])
        .block(
            Block::default()
                .title(" SELECTED VOICE WAVEFORM ")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, samples.len().saturating_sub(1) as f64])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{}", samples.len() - 1)),
                ]),
        )
        .y_axis(Axis::default().bounds([-1.0, 1.0]).labels(vec![
            Span::raw("-1"),
            Span::raw("0"),
            Span::raw("1"),
        ]));
        frame.render_widget(chart, content[1]);
    } else {
        frame.render_stateful_widget(settings, layout[3], &mut state);
    }

    let footer = Paragraph::new(vec![
        Line::from("1-8 play    +/- octave    ↑/↓ select    ←/→ edit    Tab menu    q quit"),
        Line::from(Span::styled(
            format!("MIDI: {}", panel.midi_status),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            panel.device_status.as_str(),
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, layout[4]);
}

fn midi_note_name(note: u8) -> String {
    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    format!("{}{}", NOTE_NAMES[(note % 12) as usize], note / 12 - 1)
}

fn build_audio_stream(
    device: &Device,
    config: &StreamConfig,
    synth: Arc<Mutex<synth::Synth>>,
    live_audio: Arc<LiveAudio>,
    audio_rx: Arc<Mutex<mpsc::Receiver<AudioCommand>>>,
) -> anyhow::Result<Stream> {
    let channels = config.channels as usize;
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut synth = synth
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let commands = audio_rx
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for command in commands.try_iter() {
                process_command(&mut synth, command);
            }
            drop(commands);
            for frame in data.chunks_mut(channels) {
                let output = synth.clock_and_output();
                let left = Sample::from_sample(output[0]);
                let right = Sample::from_sample(output[1]);
                live_audio.push(left);
                for (index, sample) in frame.iter_mut().enumerate() {
                    *sample = if index % 2 == 0 { left } else { right };
                }
            }
        },
        |error| eprintln!("Audio stream error: {error}"),
        None,
    )?;
    stream.play()?;
    Ok(stream)
}

fn midi_input_port_names() -> Vec<String> {
    let Ok(input) = midir::MidiInput::new("Little Weirdo FM List") else {
        return Vec::new();
    };
    input
        .ports()
        .iter()
        .filter_map(|port| input.port_name(port).ok())
        .collect()
}

fn connect_midi_port(
    port_index: Option<usize>,
    audio_tx: mpsc::Sender<AudioCommand>,
) -> (String, Option<midir::MidiInputConnection<()>>) {
    let input = match midir::MidiInput::new("Little Weirdo FM") {
        Ok(input) => input,
        Err(error) => return (format!("unavailable ({error})"), None),
    };
    let Some(port_index) = port_index else {
        return ("MIDI input: no input ports available".to_owned(), None);
    };
    let Some(port) = input.ports().into_iter().nth(port_index) else {
        return (
            "MIDI switch failed: input port is unavailable".to_owned(),
            None,
        );
    };
    let port_name = input
        .port_name(&port)
        .unwrap_or_else(|_| "MIDI input".to_owned());
    match input.connect(
        &port,
        "Little Weirdo FM",
        move |_timestamp, data, _| {
            let _ = audio_tx.send(AudioCommand::Midi(MidiMessage::from(data)));
        },
        (),
    ) {
        Ok(connection) => (
            format!("MIDI input: connected to {port_name}"),
            Some(connection),
        ),
        Err(error) => (format!("MIDI switch failed: {error}"), None),
    }
}

fn process_command(synth: &mut synth::Synth, command: AudioCommand) {
    match command {
        AudioCommand::Midi(message) => match message {
            MidiMessage::NoteOn(_, event) => synth.note_on(event.key, event.value),
            MidiMessage::NoteOff(_, event) => synth.note_off(event.key),
            _ => {}
        },
        AudioCommand::NoteOn(note, velocity) => synth.note_on(note, velocity),
        AudioCommand::NoteOff(note) => synth.note_off(note),
        AudioCommand::AdjustFm(parameter, clockwise) => {
            let mut control = synth.control();
            match parameter {
                FmParameter::CarrierWaveform => control.adjust_fm_carrier_waveform(clockwise),
                FmParameter::ModulatorWaveform => control.adjust_fm_modulator_waveform(clockwise),
                FmParameter::ModulatorRatio => control.adjust_fm_modulator_ratio(clockwise),
                FmParameter::ModulationDepth => control.adjust_fm_modulation_depth(clockwise),
            }
        }
        AudioCommand::AdjustEffect(parameter, clockwise) => {
            let mut control = synth.control();
            match parameter {
                EffectParameter::DelayTime => control.adjust_delay_time(clockwise),
                EffectParameter::DelayMix => control.adjust_delay_mix(clockwise),
                EffectParameter::DelayFeedback => control.adjust_delay_feedback(clockwise),
                EffectParameter::FlangerBaseDelay => control.adjust_flanger_base_delay(clockwise),
                EffectParameter::FlangerDepth => control.adjust_flanger_depth(clockwise),
                EffectParameter::FlangerRate => control.adjust_flanger_rate(clockwise),
                EffectParameter::FlangerMix => control.adjust_flanger_mix(clockwise),
                EffectParameter::FilterCutoff => control.adjust_filter_cutoff(clockwise),
                EffectParameter::FilterResonance => control.adjust_filter_resonance(clockwise),
                EffectParameter::FilterKind => control.cycle_filter_kind(clockwise),
                EffectParameter::DistortionKind => control.cycle_overdrive_kind(clockwise),
                EffectParameter::DistortionThreshold => {
                    control.adjust_overdrive_threshold(clockwise)
                }
            }
        }
        AudioCommand::ToggleEffect(effect) => {
            let mut control = synth.control();
            match effect {
                EffectToggle::Delay => control.toggle_delay(),
                EffectToggle::Flanger => control.toggle_flanger(),
                EffectToggle::Filter => control.toggle_filter(),
                EffectToggle::Distortion => control.toggle_overdrive(),
            }
        }
        AudioCommand::SetEnvelope(parameter, value) => {
            let mut control = synth.control();
            match parameter {
                EnvelopeParameter::Attack => {
                    control.change_all_envelopes(Some(value), None, None, None)
                }
                EnvelopeParameter::Decay => {
                    control.change_all_envelopes(None, Some(value), None, None)
                }
                EnvelopeParameter::Sustain => {
                    control.change_all_envelopes(None, None, Some(value as i16), None)
                }
                EnvelopeParameter::Release => {
                    control.change_all_envelopes(None, None, None, Some(value))
                }
            }
        }
        AudioCommand::ToggleFm(enabled) => {
            let mut control = synth.control();
            control.set_fm_enabled(enabled);
        }
        AudioCommand::ChangeVoice(voice_id) => {
            let mut control = synth.control();
            control.change_all_voices(voice_id);
        }
    }
}

fn save_patch_to_disk(panel: &SynthPanel) -> anyhow::Result<String> {
    let patch_data = serde_json::json!({
        "fm_config": {
            "enabled": panel.fm_enabled,
            "carrier_waveform": panel.carrier_waveform,
            "modulator_waveform": panel.modulator_waveform,
            "modulator_ratio": panel.modulator_ratio,
            "modulation_depth_hz": panel.modulation_depth_hz
        },
        "envelops": [{
            "attack_time": panel.attack_time_ms,
            "decay_time": panel.decay_time_ms,
            "sustain_level": panel.sustain_level as i16,
            "release_time": panel.release_time_ms
        }],
        "effects": {
            "delay": {
                "enabled": panel.delay_enabled,
                "delay_time": panel.delay_time_ms,
                "mix_percentage": panel.delay_mix,
                "feedback": panel.delay_feedback > 0,
                "feedback_percentage": panel.delay_feedback
            },
            "flanger": {
                "enabled": panel.flanger_enabled,
                "base_delay_time_ms": panel.flanger_base_delay_ms,
                "depth_ms": panel.flanger_depth_ms,
                "rate_hz": panel.flanger_rate_hz,
                "mix_percentage": panel.flanger_mix
            },
            "filter": {
                "enabled": panel.filter_enabled,
                "cutoff_frequency": panel.filter_cutoff,
                "resonance": panel.filter_resonance,
                "kind_of_filter": panel.filter_kind
            },
            "overdrive": {
                "enabled": panel.distortion_enabled,
                "kind": panel.distortion_kind,
                "threshold": panel.distortion_threshold
            }
        }
    });

    let dir = "examples/soundbank/synth/patches/original";
    fs::create_dir_all(dir)?;
    let path = format!("{}/{}.json", dir, panel.patch_name);
    fs::write(&path, serde_json::to_string_pretty(&patch_data)?)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::{computer_key_index, keyboard_note, LiveAudio, LIVE_SAMPLE_COUNT};
    use crossterm::event::KeyCode;

    #[test]
    fn number_keys_play_the_c_major_scale_from_c3_to_c4() {
        let notes = [48, 50, 52, 53, 55, 57, 59, 60];
        for (index, note) in notes.into_iter().enumerate() {
            let key = KeyCode::Char(char::from(b'1' + index as u8));
            assert_eq!(computer_key_index(key), Some(index as u8));
            assert_eq!(keyboard_note(index as u8, 0), note);
        }
        assert_eq!(computer_key_index(KeyCode::Char('9')), None);
    }

    #[test]
    fn octave_shifts_keep_the_keyboard_within_the_synth_note_range() {
        assert_eq!(keyboard_note(0, -2), 24);
        assert_eq!(keyboard_note(7, 4), 108);
    }

    #[test]
    fn live_audio_snapshot_keeps_recent_samples_in_order() {
        let live_audio = LiveAudio::new();
        live_audio.push(-0.5);
        live_audio.push(0.25);

        let snapshot = live_audio.snapshot();
        assert_eq!(snapshot.len(), LIVE_SAMPLE_COUNT);
        assert_eq!(
            snapshot[LIVE_SAMPLE_COUNT - 2],
            ((LIVE_SAMPLE_COUNT - 2) as f64, -0.5)
        );
        assert_eq!(
            snapshot[LIVE_SAMPLE_COUNT - 1],
            ((LIVE_SAMPLE_COUNT - 1) as f64, 0.25)
        );
    }
}
