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
    text::{Line},
    widgets::{Borders, Paragraph, Widget},
    Terminal, TerminalOptions,
};
use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::tzx::{
    Machine, Playlist, TzxData,
};

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    let milliseconds = duration.subsec_millis();

    format!("{:2}m {:02}s {:03}ms", minutes, seconds, milliseconds)
}

pub fn run_play(path: &Path, machine: &Machine, tzx_data: &TzxData) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide)?;
        let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::with_options(backend, TerminalOptions {
        viewport: ratatui::Viewport::Inline(6)
    })?;

    let standard_font = FIGfont::standard().unwrap();
    let figlet = standard_font.convert("rtzx").unwrap();
    terminal.insert_before(6, |buf| {
        Paragraph::new(figlet.to_string()).render(buf.area, buf);
    })?;

    let metadata_text = vec![
        Line::from(vec!["TZX file: ".into(), format!("{}", path.display()).bold()]),
        Line::from(vec!["Machine:  ".into(), format!("{:?}", machine).bold()]),
        Line::from(vec!["Header:   ".into(), format!("{}", tzx_data.header).bold()]),
    ];
    terminal.insert_before(4, |buf| {
        Paragraph::new(metadata_text).render(buf.area, buf);
    })?;

    #[cfg(debug_assertions)]
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");

    #[cfg(not(debug_assertions))]
    let mut stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");
    #[cfg(not(debug_assertions))]
    stream_handle.log_on_drop(false);
    let sink = rodio::Sink::connect_new(&stream_handle.mixer());

    let mut start_pulse_high = true;
    let mut playlist = Playlist::new(sink, machine.clone());
    for block in &tzx_data.blocks {
        playlist.append_block(block, start_pulse_high);
        start_pulse_high = block.next_block_start_pulse_high(start_pulse_high);
    }

    let mut last_block_index: usize = 0;
    let blocks_count = playlist.len_blocks();

    let ui_title = Line::from(vec![
        " Playback of ".green().bold(),
        format!("{}", path.display()).bold(),
        format!("{} ", format_duration(playlist.total_duration)).yellow(),
    ]);
    let ui_instructions = Line::from(vec![
            " Pause / Play ".into(),
            "<SPACE>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

    playlist.play();

    // UI event loop
    loop {
        let (waveform_elapsed, waveform_duration) = playlist.progress_in_current_waveform();
        let (block_elapsed, block_duration) = playlist.progress_in_current_block();
        let _ = playlist.waveforms[playlist.current_waveform_index].try_seek(waveform_elapsed);
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
                .title(ui_title.clone())
                .title_bottom(ui_instructions.clone().centered())
                .borders(Borders::ALL);
            let content = vec![
                Line::from(vec![
                    "Playing block ".into(),
                    format!("{:03}", playlist.current_block_index + 1).bold(),
                    "/".into(),
                    format!("{:03}", blocks_count).bold(),
                    format!("{:10}", format_duration(playlist.block_durations[playlist.current_block_index])).yellow(),
                    format!(": {}", playlist.blocks[playlist.current_block_index]).into(),
                ]),
                Line::from(vec![
                    format!("Waveform {}", playlist.waveforms[playlist.current_waveform_index]).into(),
                    "    ".into(),
                    format!("{}", playlist.waveforms[playlist.current_waveform_index].visualise()).fg(Color::LightYellow).bg(Color::LightBlue),
                ]),
                Line::from(vec![
                    "W: ".into(), format!("{:10}  / {:10}   ", format_duration(waveform_elapsed), format_duration(waveform_duration.saturating_sub(waveform_elapsed))).yellow(),
                    "B: ".into(), format!("{:10}  / {:10}   ", format_duration(block_elapsed), format_duration(block_duration.saturating_sub(block_elapsed))).yellow(),
                    "T: ".into(), format!("{:10}  / {:10}   ", format_duration(playlist.elapsed()), format_duration(playlist.total_duration.saturating_sub(playlist.elapsed()))).yellow(),
                    if playlist.is_paused() { "** PAUSED **".bold() } else { "".into() }
                ]),
            ];

            f.render_widget(Paragraph::new(content).block(block), chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    break;
                }

                if key.is_press() {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => playlist.toggle_pause(),
                        _ => (),
                    }
                }
            }
        }

        playlist.update_current_indices();

        while last_block_index < playlist.current_block_index {
            let last_block_text = Line::from(vec![
                "Block ".into(),
                format!("{:03}", last_block_index + 1).bold(),
                "/".into(),
                format!("{:03}", blocks_count).bold(),
                format!("{:10}", format_duration(playlist.block_durations[last_block_index])).yellow(),
                format!(": {}", playlist.blocks[last_block_index]).into(),
            ]);
            terminal.insert_before(1, |buf| {
                Paragraph::new(last_block_text).render(buf.area, buf);
            })?;
            last_block_index += 1;
        }

        if playlist.is_finished() {
            break;
        }

        // Target max 100fps
        thread::sleep(Duration::from_millis(10));
    }

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
