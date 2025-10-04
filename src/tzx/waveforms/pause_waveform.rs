use rodio::{
    ChannelCount,
    SampleRate,
    Source,
    source::SeekError,
};
use std::fmt;
use std::time::Duration;

use crate::tzx::waveforms::Waveform;

#[derive(Clone)]
pub struct PauseWaveform {
    length: u16,
    sample_index: u64,
    start_pause_pulse: bool,
}

impl PauseWaveform {
    pub fn new(length: u16) -> Self {
        return Self {
            length,
            sample_index: 0,
            start_pause_pulse: true,
        }
    }
}

impl Iterator for PauseWaveform {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None;
        }
        if self.start_pause_pulse {
            if self.sample_index == 0 {
                self.sample_index += 1;
                return Some(0.0f32);
            } else if self.sample_index < 48 {
                self.sample_index += 1;
                return Some(-1.0f32);
            }

            self.start_pause_pulse = false;
            //self.sample_index = 0;
        }
        let pause_length = self.length as u64 * 48;

        if self.sample_index < pause_length {
            self.sample_index += 1;
            return Some(0f32);
        }
        return None;
    }
}

impl Source for PauseWaveform {
    fn channels(&self) -> ChannelCount { 1 }
    fn sample_rate(&self) -> SampleRate { 48000 }
    fn current_span_len(&self) -> Option<usize> { None }

    fn total_duration(&self) -> Option<Duration> {
        return Some(Duration::from_millis(self.length as u64));
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.sample_index = if pos.as_millis() < self.length as u128 { pos.as_millis() as u64 * 48 } else { self.length as u64 * 48 - 1 };
        return Ok(());
    }
}

impl Waveform for PauseWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }
}

impl fmt::Display for PauseWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PauseWaveform:  {:6} / {:6} samples",
            self.sample_index,
            self.length as u64 * 48,
        )
    }
}
