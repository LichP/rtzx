use rodio::{
    ChannelCount,
    SampleRate,
    Source,
};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::tzx::{
    Config,
    waveforms::{Pulse, Waveform},
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct SyncWaveform {
    config: Arc<Config>,
    length_pulse_sync_first: u16,
    length_pulse_sync_second: u16,
    is_first_pulse: bool,
    current_pulse_sample_index: u32,
    pulse_first: Pulse,
    pulse_second: Pulse,
}

impl SyncWaveform {
    pub fn new(config: Arc<Config>, length_pulse_sync_first: u16, length_pulse_sync_second: u16, start_pulse_high: bool) -> Self {
        return Self {
            config: config.clone(),
            length_pulse_sync_first,
            length_pulse_sync_second,
            is_first_pulse: true,
            current_pulse_sample_index: 0,
            pulse_first: Pulse::new(config.clone(), length_pulse_sync_first, start_pulse_high),
            pulse_second: Pulse::new(config.clone(), length_pulse_sync_second, !start_pulse_high),
        }
    }
}

impl Iterator for SyncWaveform {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_first_pulse {
            let pulse_sample = self.pulse_first.next();
            if pulse_sample.is_some() {
                return pulse_sample;
            }

            self.is_first_pulse = false;
        }
        return self.pulse_second.next();
    }
}

impl Source for SyncWaveform {
    fn channels(&self) -> ChannelCount { 1 }
    fn sample_rate(&self) -> SampleRate { self.config.sample_rate }
    fn current_span_len(&self) -> Option<usize> { None }

    fn total_duration(&self) -> Option<Duration> {
        //return Some(Duration::from_secs_f64((self.pulse_first.len() as f64 + self.pulse_second.len() as f64) / 48000.0));
        return Some(self.pulse_first.duration() + self.pulse_second.duration());
    }
}

impl Waveform for SyncWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn started(&self) -> bool { !self.is_first_pulse || self.current_pulse_sample_index > 0 }
}

impl fmt::Display for SyncWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SyncWaveform")
    }
}
