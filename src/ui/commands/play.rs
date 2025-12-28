use cpal::traits::HostTrait;
use cpal::{BufferSize, SampleFormat};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode},
    cursor,
};
use figlet_rs::FIGfont;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    prelude::Direction,
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Borders, Paragraph, Widget},
    Terminal, TerminalOptions,
};
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::tzx::{
    Config, ExtendedDisplayCollector, Player, TzxData, waveforms::Waveform,
};

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    let milliseconds = duration.subsec_millis();

    format!("{:2}m {:02}s {:03}ms", minutes, seconds, milliseconds)
}

struct PlayLineCollector {
    pub lines: Vec<Line<'static>>,
}

impl ExtendedDisplayCollector for PlayLineCollector {
    fn push(&mut self, item: &dyn std::fmt::Display) {
        self.lines.push(Line::from(vec![
            "                            ".into(),
            item.to_string().into(),
        ]));
    }
}

pub fn run_play(path: &Path, config: &Config, tzx_data: &TzxData) -> io::Result<()> {
    let default_device = cpal::default_host()
        .default_output_device()
        .expect("No default audio output device is found.");
    #[allow(unused_mut)]
    let mut stream_handle = rodio::OutputStreamBuilder::from_device(default_device)
        .expect("Unable to open audio device")
        .with_buffer_size(BufferSize::Fixed(config.buffer_size()))
        .with_sample_rate(config.sample_rate)
        .with_sample_format(SampleFormat::F32)
        .open_stream_or_fallback()
        .expect("Unable to configure audio device");
    #[cfg(not(debug_assertions))]
    stream_handle.log_on_drop(false);
    let sink = rodio::Sink::connect_new(&stream_handle.mixer());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide)?;
        let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::with_options(backend, TerminalOptions {
        viewport: ratatui::Viewport::Inline(7)
    })?;

    let standard_font = FIGfont::standard().unwrap();
    let figlet = standard_font.convert("rtzx").unwrap();
    terminal.insert_before(7, |buf| {
        Paragraph::new(figlet.to_string()).render(buf.area, buf);
    })?;

    let metadata_text = vec![
        Line::from(vec!["TZX file:    ".into(), format!("{}", path.display()).bold()]),
        Line::from(vec!["Header:      ".into(), format!("{}", tzx_data.header).bold()]),
        Line::from(vec!["Platform:    ".into(), format!("{:?}", config.platform).bold()]),
        Line::from(vec!["Sample rate: ".into(), format!("{:?}", config.sample_rate).bold()]),
        Line::from(vec!["Buffer size: ".into(), format!("{:?}", config.buffer_size()).bold()]),
        Line::from(vec!["Buffer time: ".into(), format!("{:?}", config.buffer_delay()).bold()]),
    ];
    terminal.insert_before(7, |buf| {
        Paragraph::new(metadata_text).render(buf.area, buf);
    })?;

    let config = Arc::new(config.clone());

    let mut player = Player::new(sink, config.clone(), tzx_data);

    let mut last_block_index: usize = 0;

    let ui_title = Line::from(vec![
        " Playback of ".green().bold(),
        format!("{}", path.display()).bold(),
        format!(" {} ", format_duration(player.total_duration)).yellow(),
    ]);
    let ui_instructions = Line::from(vec![
            " Pause / Play ".into(),
            "<SPACE>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
            " Rewind to previous block ".into(),
            "<←>".blue().bold().into(),
            " Skip to next block ".into(),
            "<→> ".blue().bold().into(),
        ]);

    player.play();

    let mut playback_render = true;

    // UI event loop
    loop {
        if playback_render {
            if !player.is_paused() {
                let (waveform_elapsed, _) = player.progress_in_current_waveform();
                let _ = player.waveforms[player.current_waveform_index].try_seek(waveform_elapsed);
            }
            render_playback_pane(&mut terminal, ui_title.clone(), ui_instructions.clone(), &player)?;
        }

        playback_render = !player.is_paused();

        if event::poll(std::time::Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    break;
                }

                if key.is_press() {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => player.toggle_pause(),
                        KeyCode::Left => if player.current_block_index > 0 {
                            if player.block_durations[player.current_block_index - 1] == Duration::ZERO && player.current_block_index > 1 {
                                player.seek_block(player.current_block_index - 2);
                            } else {
                                player.seek_block(player.current_block_index - 1);
                            }
                            playback_render = true;
                        },
                        KeyCode::Right => if player.current_block_index < player.len_blocks() - 1 {
                            player.seek_block(player.current_block_index + 1);
                            playback_render = true;
                        },
                        _ => (),
                    }
                }
            }
        }

        player.tick();

        while last_block_index < player.current_block_index {
            let last_block_text = Line::from(vec![
                "Block ".into(),
                format!("{:03}", last_block_index + 1).bold(),
                "/".into(),
                format!("{:03}", player.len_blocks()).bold(),
                format!("{:10}", format_duration(player.block_durations[last_block_index])).yellow(),
                format!(": {}", player.blocks[last_block_index]).into(),
            ]);
            terminal.insert_before(1, |buf| {
                Paragraph::new(last_block_text).render(buf.area, buf);
            })?;

            let mut extended_lines_collector = PlayLineCollector { lines: vec![] };
            player.blocks[last_block_index].extended_display(&mut extended_lines_collector);
            for line in extended_lines_collector.lines {
                terminal.insert_before(1, |buf| {
                    Paragraph::new(line).render(buf.area, buf);
                })?;
            }

            last_block_index += 1;
        }

        if !player.is_paused() && player.is_finished() {
            break;
        }

        // Target max 100fps
        thread::sleep(Duration::from_millis(10));
    }

    player.finish();

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        cursor::Show,
    )?;
    terminal.show_cursor()?;
    println!();
    println!();
    return Ok(());
}

pub fn render_playback_pane(terminal: &mut Terminal<CrosstermBackend<&mut std::io::Stdout>>, ui_title: Line<'_>, ui_instructions: Line<'_>, player: &Player) -> io::Result<()> {
    let blocks_count = player.len_blocks();
    let (waveform_elapsed, waveform_duration) = player.progress_in_current_waveform();
    let (block_elapsed, block_duration) = player.progress_in_current_block();

    terminal.draw(|f| {
        let area = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // top row
                Constraint::Min(0),    // everything else
            ])
            .split(area);

        let block = ratatui::widgets::Block::default()
            .title(ui_title)
            .title_bottom(ui_instructions.centered())
            .borders(Borders::ALL);

        let waveform_data_line = render_waveform_data_line(&player.waveforms[player.current_waveform_index]);

        let content = vec![
            Line::from(vec![
                "Playing block ".into(),
                format!("{:03}", player.current_block_index + 1).bold(),
                "/".into(),
                format!("{:03}", blocks_count).bold(),
                format!("{:10}", format_duration(player.block_durations[player.current_block_index])).yellow(),
                format!(": {}", player.blocks[player.current_block_index]).into(),
            ]),
            Line::from(vec![
                format!("Waveform {}", player.waveforms[player.current_waveform_index]).into(),
            ]),
            waveform_data_line,
            Line::from(vec![
                "W: ".into(), format!("{:10}  / {:10}   ", format_duration(waveform_elapsed), format_duration(waveform_duration.saturating_sub(waveform_elapsed))).yellow(),
                "B: ".into(), format!("{:10}  / {:10}   ", format_duration(block_elapsed), format_duration(block_duration.saturating_sub(block_elapsed))).yellow(),
                "T: ".into(), format!("{:10}  / {:10}   ", format_duration(player.elapsed()), format_duration(player.total_duration.saturating_sub(player.elapsed()))).yellow(),
                if player.is_paused() { "** PAUSED **".bold() } else { "".into() }
            ]),
        ];

        f.render_widget(Paragraph::new(content).block(block), chunks[1]);
    })?;

    return Ok(());
}

fn render_waveform_data_line(waveform: &Box<dyn Waveform + Send>) -> Line<'_> {
    let mut spans: Vec<Span<'_>> = Vec::new();
    spans.push(format!("{}", waveform.visualise(64)).fg(Color::LightYellow).bg(Color::LightBlue));

    if let Some(payload_with_position) = waveform.payload_with_position() {
        spans.push(format!("  {:04x}: ", payload_with_position.current_row_address()).into());

        for (i, byte) in payload_with_position.current_row_bytes().iter().enumerate() {
            if i % 16 == 8 {
                spans.push(" ".into());
            }

            let byte_formatted: Span<'_> = format!("{:02x} ", byte).into();
            spans.push(if payload_with_position.current_byte_index % 16 == i {byte_formatted.yellow() } else { byte_formatted });
        }

        spans.push("  |".into());
        for (i, byte) in payload_with_position.current_row_bytes().iter().enumerate() {
            let byte_ascii: Span<'_> = to_ascii_or_dot(*byte).to_string().into();
            spans.push(if payload_with_position.current_byte_index % 16 == i {byte_ascii.yellow() } else { byte_ascii });
        }
        spans.push("|".into());
    }

    return Line::from(spans);
}

fn to_ascii_or_dot(byte: u8) -> char {
    let c = byte as char;
    if c.is_ascii_graphic() || c == ' ' {
        c
    } else {
        '.'
    }
}
