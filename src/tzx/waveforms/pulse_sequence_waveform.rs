use rodio::{
    ChannelCount,
    SampleRate,
    Source,
    source::SeekError,
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
pub struct PulseSequenceWaveform {
    config: Arc<Config>,
    total_length: usize,
    current_pulse_index: usize,
    current_pulse_sample_index: u32,
    pulses: Vec<Pulse>,
}

impl PulseSequenceWaveform {
    pub fn new(config: Arc<Config>, pulse_lengths: &Vec<u16>,  start_pulse_high: bool) -> Self {
        let mut current_pulse_high = start_pulse_high;
        let mut pulses: Vec<Pulse> = vec![];
        let mut total_length: usize = 0;

        for pulse_length in pulse_lengths {
            pulses.push(Pulse::new(config.clone(), *pulse_length, current_pulse_high));
            total_length += *pulse_length as usize;
            current_pulse_high = !current_pulse_high;
        }

        return Self {
            config,
            total_length,
            current_pulse_index: 0,
            current_pulse_sample_index: 0,
            pulses: pulses,
        }
    }
}

impl Iterator for PulseSequenceWaveform {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pulse_index < self.pulses.len() {
            let pulse_sample = self.pulses[self.current_pulse_index].next();
            if pulse_sample.is_some() {
                return pulse_sample;
            }

            self.current_pulse_index += 1;
            if self.current_pulse_index < self.pulses.len() {
                return self.pulses[self.current_pulse_index].next()
            }
        }
        return None;
    }
}

impl Source for PulseSequenceWaveform {
    fn channels(&self) -> ChannelCount { 1 }
    fn sample_rate(&self) -> SampleRate { self.config.sample_rate }
    fn current_span_len(&self) -> Option<usize> { None }

    fn total_duration(&self) -> Option<Duration> {
        let mut duration = Duration::ZERO;
        for pulse in &self.pulses {
            duration += pulse.duration();
        }
        return Some(duration);
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let samples = (pos.as_secs_f64() * self.config.sample_rate as f64).round() as u128;
        let mut pulse_samples = 0;
        self.current_pulse_index = 0;
        while pulse_samples < samples && self.current_pulse_index < self.pulses.len() {
            pulse_samples += self.pulses[self.current_pulse_index].len() as u128;
            self.current_pulse_index += 1;
        }
        return Ok(());
    }
}

impl Waveform for PulseSequenceWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn started(&self) -> bool { self.current_pulse_index > 0 || self.current_pulse_sample_index > 0 }
}

impl fmt::Display for PulseSequenceWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PulseSequenceWaveform: {:5} / {:5} pulses",
            self.current_pulse_index,
            self.pulses.len(),
        )
    }
}
