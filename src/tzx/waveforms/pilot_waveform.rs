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
    current_pulse: Pulse,
    current_pulse_index: u32,
    current_pulse_sample_index: u32,
}

impl PilotWaveform {
    pub fn new(config: Arc<Config>, length_pulse: u16, length_tone: u16, start_pulse_high: bool) -> Self {
        let current_pulse = Pulse::new(config.clone(), length_pulse, start_pulse_high);

        return Self {
            config,
            length_pulse,
            length_tone,
            current_pulse,
            current_pulse_index: 0,
            current_pulse_sample_index: 0,
        }
    }

    fn update_pulse(&mut self) {
        self.current_pulse.high = !self.current_pulse.high;
    }
}

impl Iterator for PilotWaveform {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pulse_index < self.length_tone as u32 {
            let pulse_sample = self.current_pulse.get_next_sample(self.current_pulse_sample_index);
            if pulse_sample.is_some() {
                self.current_pulse_sample_index += 1;
                return pulse_sample;
            }

            self.current_pulse_index += 1;
            self.current_pulse_sample_index = 0;
            if self.current_pulse_index < self.length_tone as u32 {
                self.update_pulse();
                return self.next()
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
        if self.length_tone == 0 {
            return None;
        }
        return Some(self.current_pulse.duration() * self.length_tone as u32);
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let samples = (pos.as_secs_f64() * self.config.sample_rate as f64).round() as u32;
        let mut pulse_samples = 0;
        self.current_pulse_index = 0;
        self.current_pulse_sample_index = 0;
        while pulse_samples < samples && self.current_pulse_index < self.length_tone as u32 {
            pulse_samples += self.current_pulse.len();
            if pulse_samples > samples {
                self.current_pulse_sample_index = pulse_samples - samples;
            } else {
                self.current_pulse_index += 1;
                self.update_pulse()
            }
        }
        return Ok(());
    }
}

impl Waveform for PilotWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn started(&self) -> bool { self.current_pulse_index > 0 || self.current_pulse_sample_index > 0 }

    fn visualise(&self, pulse_string_length: usize) -> String {
        let mut pulse_string = "".to_string();
        let mut pulse_index = self.current_pulse_index;
        let mut current_high = self.current_pulse.high;
        let mut current_char: char;
        while pulse_string.chars().count() < pulse_string_length && pulse_index < self.length_tone as u32 {
            current_char = if current_high { '\u{2588}' } else { ' ' };
            pulse_string.push(current_char);
            pulse_string.push(current_char);
            pulse_index += 1;
            current_high = !current_high
        }
        return pulse_string;
    }
}

impl fmt::Display for PilotWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PilotWaveform:  {:6} / {:6} pulses",
            self.current_pulse_index + 1,
            self.length_tone,
        )
    }
}
