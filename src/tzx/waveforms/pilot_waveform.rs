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
pub struct PilotWaveform {
    config: Arc<Config>,
    length_pulse: u16,
    length_tone: u16,
    pulses: Vec<Pulse>,
    current_pulse_index: u32,
    current_pulse_sample_index: u32,
}

impl PilotWaveform {
    pub fn new(config: Arc<Config>, length_pulse: u16, length_tone: u16, start_pulse_high: bool) -> Self {
        let mut current_pulse_high = start_pulse_high;
        let mut pulses: Vec<Pulse> = vec![];
        for _ in 0..length_tone {
            pulses.push(Pulse::new(config.clone(), length_pulse, current_pulse_high));
            current_pulse_high = !current_pulse_high;
        }

        return Self {
            config,
            length_pulse,
            length_tone,
            current_pulse_index: 0,
            pulses: pulses,
            current_pulse_sample_index: 0,
        }
    }
}

impl Iterator for PilotWaveform {
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
impl Source for PilotWaveform {
    fn channels(&self) -> ChannelCount { 1 }
    fn sample_rate(&self) -> SampleRate { self.config.sample_rate }
    fn current_span_len(&self) -> Option<usize> { None }

    fn total_duration(&self) -> Option<Duration> {
        if self.pulses.len() == 0 {
            return None;
        }
        return Some(self.pulses[0].duration() * self.pulses.len() as u32);
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

impl Waveform for PilotWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn started(&self) -> bool { self.current_pulse_index > 0 || self.current_pulse_sample_index > 0 }

    fn visualise(&self) -> String {
        let mut pulse_string = "".to_string();
        let mut pulse_index = self.current_pulse_index;
        let mut current_char: char;
        while pulse_string.chars().count() < 32 && pulse_index < self.pulses.len() {
            current_char = if self.pulses[pulse_index].high { '\u{2588}' } else { ' ' };
            pulse_string.push(current_char);
            pulse_string.push(current_char);
            pulse_index += 1;
        }
        return pulse_string;
    }
}

impl fmt::Display for PilotWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PilotWaveform:  {:6} / {:6} pulses",
            self.current_pulse_index,
            self.pulses.len(),
        )
    }
}
