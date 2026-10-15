use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, StreamConfig};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use little_weirdo::synth::data::patches::{BoxedPatch, BoxedPatches};
use little_weirdo::synth::{
    self,
    data::waveforms::{BoxedWaveform, BoxedWaveforms},
};
use midi_control::MidiMessage;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{
    fs,
    io::{self, Stdout},
    sync::{mpsc, Arc},
    time::Duration,
};

#[derive(Clone, Copy)]
enum FmParameter {
    CarrierWaveform,
    ModulatorWaveform,
    ModulatorRatio,
    ModulationDepth,
}

enum AudioCommand {
    Midi(MidiMessage),
    AdjustFm(FmParameter, bool),
}

struct FmPanel {
    carrier_waveform: u8,
    modulator_waveform: u8,
    modulator_ratio: u8,
    modulation_depth_hz: u16,
    selected: usize,
    waveform_count: u8,
    midi_status: String,
}

fn main() -> anyhow::Result<()> {
    let (device, config) = setup_device()?;
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

    let patch: synth::patch::Patch = serde_json::from_slice(include_bytes!(
        "soundbank/synth/patches/original/fm_basic.json"
    ))?;
    let fm = patch.fm_config;
    let patch = BoxedPatch::new(patch);
    let mut patches = BoxedPatches::new();
    patches.add(patch);
    let patches = Arc::new(patches);

    let mut synth = synth::Synth::new(
        config.sample_rate.min(u16::MAX as u32) as u16,
        0,
        Arc::clone(&patches),
        Arc::clone(&waveforms),
    );
    let (audio_tx, audio_rx) = mpsc::channel::<AudioCommand>();
    let (midi_status, midi_connection) = connect_midi(audio_tx.clone());

    let channels = config.channels as usize;
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for command in audio_rx.try_iter() {
                process_command(&mut synth, command);
            }
            for frame in data.chunks_mut(channels) {
                let output = synth.clock_and_output();
                let left = Sample::from_sample(output[0]);
                let right = Sample::from_sample(output[1]);
                for (index, sample) in frame.iter_mut().enumerate() {
                    *sample = if index % 2 == 0 { left } else { right };
                }
            }
        },
        |error| eprintln!("Audio stream error: {error}"),
        None,
    )?;
    stream.play()?;

    let mut panel = FmPanel {
        carrier_waveform: fm.carrier_waveform,
        modulator_waveform: fm.modulator_waveform,
        modulator_ratio: fm.modulator_ratio.max(1),
        modulation_depth_hz: fm.modulation_depth_hz,
        selected: 0,
        waveform_count,
        midi_status,
    };
    run_terminal_ui(&mut panel, &audio_tx)?;

    drop(midi_connection);
    drop(stream);
    Ok(())
}

fn run_terminal_ui(
    panel: &mut FmPanel,
    audio_tx: &mpsc::Sender<AudioCommand>,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = event_loop(&mut terminal, panel, audio_tx);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    panel: &mut FmPanel,
    audio_tx: &mpsc::Sender<AudioCommand>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| draw_panel(frame, panel))?;
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break,
            KeyCode::Up | KeyCode::Char('k') => panel.selected = panel.selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => panel.selected = (panel.selected + 1).min(3),
            KeyCode::Left | KeyCode::Char('h') => adjust_selected(panel, audio_tx, false)?,
            KeyCode::Right | KeyCode::Char('l') => adjust_selected(panel, audio_tx, true)?,
            _ => {}
        }
    }
    Ok(())
}

fn adjust_selected(
    panel: &mut FmPanel,
    audio_tx: &mpsc::Sender<AudioCommand>,
    clockwise: bool,
) -> anyhow::Result<()> {
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
        _ => {
            panel.modulation_depth_hz = if clockwise {
                panel.modulation_depth_hz.saturating_add(10)
            } else {
                panel.modulation_depth_hz.saturating_sub(10)
            };
            FmParameter::ModulationDepth
        }
    };
    audio_tx.send(AudioCommand::AdjustFm(parameter, clockwise))?;
    Ok(())
}

fn cycle_waveform(current: u8, count: u8, clockwise: bool) -> u8 {
    if clockwise {
        current.wrapping_add(1) % count
    } else {
        current.wrapping_add(count).wrapping_sub(1) % count
    }
}

fn draw_panel(frame: &mut ratatui::Frame<'_>, panel: &FmPanel) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "LITTLE WEIRDO",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  /  FM PATCH EDITOR"),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, layout[0]);

    let values = [
        format!("Carrier waveform       {:03}", panel.carrier_waveform),
        format!("Modulator waveform     {:03}", panel.modulator_waveform),
        format!("Modulator ratio        {} : 1", panel.modulator_ratio),
        format!("Modulation depth       {} Hz", panel.modulation_depth_hz),
    ];
    let items = values
        .iter()
        .map(|value| ListItem::new(value.as_str()))
        .collect::<Vec<_>>();
    let settings = List::new(items)
        .block(
            Block::default()
                .title(" FM PARAMETERS ")
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
    frame.render_stateful_widget(settings, layout[1], &mut state);

    let footer = Paragraph::new(vec![
        Line::from("↑/↓ select    ←/→ change    q quit"),
        Line::from(Span::styled(
            format!("MIDI: {}", panel.midi_status),
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, layout[2]);
}

fn setup_device() -> anyhow::Result<(Device, StreamConfig)> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default audio output device is unavailable"))?;
    let config = device.default_output_config()?.into();
    Ok((device, config))
}

fn connect_midi(
    audio_tx: mpsc::Sender<AudioCommand>,
) -> (String, Option<midir::MidiInputConnection<()>>) {
    let input = match midir::MidiInput::new("Little Weirdo FM") {
        Ok(input) => input,
        Err(error) => return (format!("unavailable ({error})"), None),
    };
    let Some(port) = find_port(&input) else {
        return ("not connected; no IAC MIDI port found".to_owned(), None);
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
        Ok(connection) => (format!("connected: {port_name}"), Some(connection)),
        Err(error) => (format!("connection failed ({error})"), None),
    }
}

fn process_command(synth: &mut synth::Synth, command: AudioCommand) {
    match command {
        AudioCommand::Midi(message) => match message {
            MidiMessage::NoteOn(_, event) => synth.note_on(event.key, event.value),
            MidiMessage::NoteOff(_, event) => synth.note_off(event.key),
            _ => {}
        },
        AudioCommand::AdjustFm(parameter, clockwise) => {
            let mut control = synth.control();
            match parameter {
                FmParameter::CarrierWaveform => control.adjust_fm_carrier_waveform(clockwise),
                FmParameter::ModulatorWaveform => control.adjust_fm_modulator_waveform(clockwise),
                FmParameter::ModulatorRatio => control.adjust_fm_modulator_ratio(clockwise),
                FmParameter::ModulationDepth => control.adjust_fm_modulation_depth(clockwise),
            }
        }
    }
}

fn find_port<T>(midi_io: &T) -> Option<T::Port>
where
    T: midir::MidiIO,
{
    midi_io.ports().into_iter().find(|port| {
        midi_io
            .port_name(port)
            .map(|name| name.contains("IAC"))
            .unwrap_or(false)
    })
}
