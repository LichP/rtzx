use rodio::{Sink, Source};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::tzx::{
    Config,
    blocks::Block,
    TzxData,
    waveforms::Waveform,
};

pub struct Player<'a> {
    config: Arc<Config>,
    sink: Sink,
    tzx_data: &'a TzxData,
    pub blocks: Vec<Box<dyn Block>>,
    pub block_durations: Vec<std::time::Duration>,
    pub waveforms: Vec<Box<dyn Waveform + Send>>,
    pub waveforms_original: Vec<Box<dyn Waveform + Send>>,
    pub waveform_durations: Vec<std::time::Duration>,
    pub total_duration: Duration,
    pub current_block_index: usize,
    pub current_waveform_index: usize,
    start_time: Option<Instant>,
    playback_duration: Duration,
    waveform_queued_index: usize,
    is_paused: bool,
    is_seeking: bool,
}

impl<'a> Player<'a> {
    pub fn new(sink: Sink, config: Arc<Config>, tzx_data: &'a TzxData) -> Player<'a> {
        sink.pause();

        let mut player = Player {
            config,
            sink,
            tzx_data,
            blocks: vec![],
            block_durations: vec![],
            waveforms: vec![],
            waveforms_original: vec![],
            waveform_durations: vec![],
            total_duration: Duration::ZERO,
            current_block_index: 0,
            current_waveform_index: 0,
            start_time: None,
            playback_duration: Duration::ZERO,
            waveform_queued_index: 0,
            is_paused: true,
            is_seeking: false,
        };

        player.read_blocks(true);

        return player;
    }

    fn read_blocks(&mut self, start_pulse_high: bool) {
        let mut current_start_pulse_high = start_pulse_high;
        let blocks = self.tzx_data.blocks.clone();
        for block in blocks {
            self.append_block(&block, current_start_pulse_high);
            current_start_pulse_high = block.next_block_start_pulse_high(self.config.clone(), current_start_pulse_high);
        }
    }

    fn append_block(&mut self, block: &Box<dyn Block>, start_pulse_high: bool) {
        let mut block_duration = Duration::ZERO;
        let waveforms = block.get_waveforms(self.config.clone(), start_pulse_high);

        for waveform in waveforms {
            let waveform_duration = match waveform.total_duration() {
                Some(duration) => duration,
                None => Duration::ZERO,
            };
            block_duration += waveform_duration;
            self.total_duration += waveform_duration;
            self.waveforms.push(waveform.clone());
            self.waveform_durations.push(waveform_duration);
            if self.waveform_queued_index < self.current_waveform_index + 1000 {
                self.waveform_queued_index += 1;
                let source: Box<dyn Source + Send> = waveform.clone();
                self.sink.append(source);
            }
            self.waveforms_original.push(waveform);
        }

        self.blocks.push(block.clone());
        self.block_durations.push(block_duration);
    }

    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start_time {
            if self.is_paused {
                self.playback_duration
            } else {
                self.playback_duration + start.elapsed()
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

    pub fn tick(&mut self) -> () {
        if !self.is_paused && !self.is_finished() {
            // Check to see if we need to queue any more data
            while self.waveform_queued_index < self.waveforms.len() - 1 && (self.waveform_queued_index - self.current_waveform_index < 1000) {
                self.waveform_queued_index += 1;
                let source: Box<dyn Source + Send> = self.waveforms_original[self.waveform_queued_index].clone();
                self.sink.append(source);
            }
            self.update_current_indices();
        }
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
                self.playback_duration += start.elapsed();
            }
            self.is_paused = true;
        }
    }

    pub fn play(&mut self) {
        if self.is_paused {
            if self.start_time.is_some() {
                let (waveform_playback_duration, _) = self.progress_in_current_waveform();
                let _ = self.sink.try_seek(waveform_playback_duration);
            }
            self.sink.play();
            self.start_time = Instant::now().checked_add(self.config.buffer_delay());
            // self.start_time = Some(Instant::now());
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

    pub fn seek_block(&mut self, block_index: usize) {
        // Do not attempt to seek while already seeking.
        if self.is_seeking { return; }
        self.is_seeking = true;

        // Pause playback before seeking. We don't auto resume afterwards, the user is expected
        // to manually unpause once they have seeked to the desired block.
        // We also stop and clear the sink ready for rebuild.
        self.is_paused = true;
        self.sink.stop();
        self.sink.clear();

        // Ensure seeked block index is bounded above (usize datatype ensures lower bound).
        let mut bounded_block_index = block_index;
        if bounded_block_index > self.blocks.len() {
            bounded_block_index = self.blocks.len() - 1;
        }

        // Recalculate playback_duration from durations of all blocks prior to seeked block.
        let mut new_playback_duration = Duration::ZERO;
        for (i, &block_duration) in self.block_durations.iter().enumerate() {
            if i == bounded_block_index {
                break;
            }
            new_playback_duration += block_duration;
        }
        self.playback_duration = new_playback_duration;

        // Recalculate indices based on new playback_duration
        self.update_current_indices();

        // Rebuild sink using all waveforms from the current index onwards.
        self.waveform_queued_index = self.current_waveform_index;
        for i in self.current_waveform_index..self.waveforms.len() {
            if self.waveforms[i].started() {
                self.waveforms[i] = self.waveforms_original[i].clone();
            }
            if self.waveform_queued_index < self.current_waveform_index + 1000 {
                self.waveform_queued_index += 1;
                let source: Box<dyn Source + Send> = self.waveforms[i].clone();
                self.sink.append(source);
            }
        }

        self.is_seeking = false;
    }

    pub fn finish(&mut self) {
        // Allow enough time for buffered audio to be output
        thread::sleep(self.config.buffer_delay() + Duration::from_millis(10));
        self.sink.stop();
    }

    pub fn len_blocks(&self) -> usize { self.blocks.len() }

    pub fn is_finished(&self) -> bool { self.sink.empty() }
    pub fn is_paused(&self) -> bool { self.is_paused }
}
