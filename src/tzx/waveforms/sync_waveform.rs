use rodio::{
    ChannelCount,
    SampleRate,
    Source,
};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::tzx::{
    Machine,
    waveforms::{Pulse, Waveform},
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct SyncWaveform {
    length_pulse_sync_first: u16,
    length_pulse_sync_second: u16,
    is_first_pulse: bool,
    pulse_first: Pulse,
    pulse_second: Pulse,
}

impl SyncWaveform {
    pub fn new(machine: Arc<Machine>, length_pulse_sync_first: u16, length_pulse_sync_second: u16, start_pulse_high: bool) -> Self {
        return Self {
            length_pulse_sync_first,
            length_pulse_sync_second,
            is_first_pulse: true,
            pulse_first: Pulse::new(machine.clone(), length_pulse_sync_first, start_pulse_high),
            pulse_second: Pulse::new(machine.clone(), length_pulse_sync_second, !start_pulse_high),
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
    fn sample_rate(&self) -> SampleRate { 48000 }
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
}

impl fmt::Display for SyncWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SyncWaveform")
    }
}
