use bitvec::prelude::*;
use rodio::{
    ChannelCount,
    SampleRate,
    Source,
    source::SeekError,
};
use std::sync::Arc;
use std::fmt;
use std::time::Duration;

use crate::tzx::{
    Config,
    data::DataPayload,
    waveforms::{Pulse,Waveform}
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct DirectWaveform {
    config: Arc<Config>,
    length_sample: u16,
    payload: DataPayload,
    current_pulse: Pulse,
    current_pulse_index: usize,
    current_pulse_sample_index: usize,
}

impl DirectWaveform {
    pub fn new(config: Arc<Config>, length_sample: u16, payload: DataPayload) -> Self {
        let first_bit = payload.data.view_bits::<Msb0>()[0];
        let current_pulse = Pulse::new(config.clone(), length_sample, first_bit);

        return Self {
            config,
            length_sample,
            payload,
            current_pulse,
            current_pulse_index: 0,
            current_pulse_sample_index: 0,
        }
    }

    pub fn last_pulse_high(&self) -> bool {
        self.payload.data.view_bits::<Msb0>()[self.payload.total_bits() - 1]
    }

    fn has_data_remaining(&self) -> bool { self.current_pulse_index < self.payload.total_bits() }

    fn update_pulse(&mut self) {
        if self.has_data_remaining() {
            self.current_pulse.high = self.payload.data.view_bits::<Msb0>()[self.current_pulse_index];
        }
    }
}

impl Iterator for DirectWaveform {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_data_remaining() {
            let pulse_sample: Option<Self::Item> = self.current_pulse.get_next_sample(self.current_pulse_sample_index as u32);
            if pulse_sample.is_some() {
                self.current_pulse_sample_index += 1;
                return pulse_sample;
            }

            self.current_pulse_index += 1;
            self.current_pulse_sample_index = 0;
            if self.has_data_remaining() {
                self.update_pulse();
                return self.next();
            }
        }
        return None;
    }
}

impl Source for DirectWaveform {
    fn channels(&self) -> ChannelCount { 1 }
    fn sample_rate(&self) -> SampleRate { self.config.sample_rate }
    fn current_span_len(&self) -> Option<usize> { None }

    fn total_duration(&self) -> Option<Duration> { Some(self.current_pulse.duration() * self.payload.total_bits() as u32) }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        if self.payload.len() == 0 { return Ok(()) }
        let samples = (pos.as_secs_f64() * self.config.sample_rate as f64).round() as usize;

        let byte_index = ((self.payload.len() - 1) as f32 * pos.div_duration_f32(self.total_duration().unwrap())) as usize;

        // Calculate the number of samples in the start to estimated byte position range.
        let mut pulse_samples: usize = 0;
        let bit_counts_to_index = self.payload.bit_counts_for_range(0..byte_index).unwrap();
        pulse_samples += self.current_pulse.len() as usize * bit_counts_to_index.total;

        self.current_pulse_index = byte_index * 8;
        self.current_pulse_sample_index = 0;
        while pulse_samples < samples && self.has_data_remaining() {
            pulse_samples += self.current_pulse.len() as usize;
            if pulse_samples > samples {
                self.current_pulse_sample_index = pulse_samples - samples;
            } else {
                self.current_pulse_index += 1;
                self.update_pulse();
            }
        }
        return Ok(());
    }
}

impl Waveform for DirectWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn started(&self) -> bool { self.current_pulse_index > 0 || self.current_pulse_sample_index > 0 }

    fn visualise(&self, pulse_string_length: usize) -> String {
        let mut pulse_string = "".to_string();
        let mut pulse_index = self.current_pulse_index;
        let mut current_char: char;
        while pulse_string.chars().count() < pulse_string_length && pulse_index < self.payload.total_bits() {
            current_char = if self.payload.data.view_bits::<Msb0>()[pulse_index] { '\u{2588}' } else { ' ' };
            pulse_string.push(current_char);
            pulse_index += 1;
        }
        return pulse_string;
    }
}

impl fmt::Display for DirectWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DirectWaveform: {:6} / {:6} pulses",
            self.current_pulse_index,
            self.payload.total_bits(),
        )
    }
}
