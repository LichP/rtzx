use rodio::{Sink, Source};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::tzx::{
    Machine,
    blocks::Block,
    waveforms::Waveform,
};

pub struct Playlist {
    sink: Sink,
    machine: Arc<Machine>,
    pub blocks: Vec<Box<dyn Block>>,
    pub block_durations: Vec<std::time::Duration>,
    pub waveforms: Vec<Box<dyn Waveform + Send>>,
    pub waveform_durations: Vec<std::time::Duration>,
    pub total_duration: Duration,
    pub current_block_index: usize,
    pub current_waveform_index: usize,
    start_time: Option<Instant>,
    paused_time: Duration,
    is_paused: bool,
}

impl Playlist {
    pub fn new(sink: Sink, machine: Machine) -> Playlist {
        sink.pause();

        return Playlist {
            sink,
            machine: Arc::new(machine),
            blocks: vec![],
            block_durations: vec![],
            waveforms: vec![],
            waveform_durations: vec![],
            total_duration: Duration::ZERO,
            current_block_index: 0,
            current_waveform_index: 0,
            start_time: None,
            paused_time: Duration::ZERO,
            is_paused: true,
        }
    }

    pub fn append_block(&mut self, block: &Box<dyn Block>, start_pulse_high: bool) {
        let mut block_duration = Duration::ZERO;
        let waveforms = block.get_waveforms(self.machine.clone(), start_pulse_high);

        for waveform in waveforms {
            let source: Box<dyn Source + Send> = waveform.clone();
            let source_duration = match source.total_duration() {
                Some(duration) => duration,
                None => Duration::ZERO,
            };
            block_duration += source_duration;
            self.total_duration += source_duration;
            self.waveforms.push(waveform);
            self.waveform_durations.push(source_duration);
            self.sink.append(source);
        }

        self.blocks.push(block.clone());
        self.block_durations.push(block_duration);
    }

    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start_time {
            if self.is_paused {
                self.paused_time
            } else {
                self.paused_time + start.elapsed()
            }
        } else {
            Duration::ZERO
        }
    }

    pub fn progress_in_current_block(&self) -> (Duration, Duration) {
        let mut remaining = self.elapsed();
        let mut index = 0;

        for (i, &block_duration) in self.block_durations.iter().enumerate() {
            if remaining < block_duration {
                index = i;
                break;
            }
            remaining -= block_duration;
        }

        (remaining, self.block_durations[index])
    }

    pub fn progress_in_current_waveform(&self) -> (Duration, Duration) {
        let mut remaining = self.elapsed();
        let mut index = 0;

        for (i, &waveform_duration) in self.waveform_durations.iter().enumerate() {
            if remaining < waveform_duration {
                index = i;
                break;
            }
            remaining -= waveform_duration;
        }

        (remaining, self.waveform_durations[index])
    }

    pub fn update_current_indices(&mut self) {
        let mut remaining = self.elapsed();

        for (i, &block_duration) in self.block_durations.iter().enumerate() {
            if remaining < block_duration {
                self.current_block_index = i;
                break;
            }
            remaining -= block_duration;
        }

        remaining = self.elapsed();

        for (i, &waveform_duration) in self.waveform_durations.iter().enumerate() {
            if remaining < waveform_duration {
                self.current_waveform_index = i;
                break;
            }
            remaining -= waveform_duration;
        }
    }

    pub fn pause(&mut self) {
        if !self.is_paused {
            self.sink.pause();
            if let Some(start) = self.start_time {
                self.paused_time += start.elapsed();
            }
            self.is_paused = true;
        }
    }

    pub fn play(&mut self) {
        if self.is_paused {
            self.sink.play();
            self.start_time = Some(Instant::now());
            self.is_paused = false;
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.is_paused {
            self.play();
        } else {
            self.pause();
        }
    }

    pub fn len_blocks(&self) -> usize { self.blocks.len() }
    pub fn len_waveforms(&self) -> usize { self.waveforms.len() }

    pub fn is_finished(&self) -> bool { self.sink.empty() }
    pub fn is_paused(&self) -> bool { self.is_paused }
}
