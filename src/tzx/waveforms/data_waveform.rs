use bitvec::prelude::*;
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
    data::{DataPayload, DataPayloadWithPosition},
    waveforms::{Pulse, Waveform},
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct DataWaveform {
    config: Arc<Config>,
    length_pulse_zero: u16,
    length_pulse_one: u16,
    payload: DataPayload,
    start_pulse_high: bool,
    current_pulse: Pulse,
    current_pulse_index: usize,
    current_pulse_sample_index: usize,
}

impl DataWaveform {
    pub fn new(config: Arc<Config>, length_pulse_zero: u16, length_pulse_one: u16, payload: DataPayload, start_pulse_high: bool) -> Self {
        let first_bit = payload.data.view_bits::<Msb0>()[0];
        let first_bit_length = if first_bit { length_pulse_one } else { length_pulse_zero };

        let current_pulse = Pulse::new(config.clone(), first_bit_length, start_pulse_high);

        return Self {
            config,
            length_pulse_zero,
            length_pulse_one,
            payload: payload,
            start_pulse_high,
            current_pulse,
            current_pulse_index: 0,
            current_pulse_sample_index: 0,
        }
    }

    fn has_data_remaining(&self) -> bool { self.current_pulse_index < self.payload.total_bits() * 2 }

    fn update_pulse(&mut self) {
        if self.has_data_remaining() {
            let current_bit = self.payload.data.view_bits::<Msb0>()[self.current_pulse_index as usize / 2];
            self.current_pulse.length = if current_bit { self.length_pulse_one } else { self.length_pulse_zero };
            self.current_pulse.high = !((self.current_pulse_index % 2 == 0) ^ self.start_pulse_high)
        }
    }
}

impl fmt::Display for DataWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bit_counts = self.payload.bit_counts();
        write!(f, "DataWaveform:   {:6} / {:6} pulses (0+1: {}+{})",
            self.current_pulse_index,
            bit_counts.total * 2,
            bit_counts.zeros,
            bit_counts.ones,
        )
    }
}

impl Iterator for DataWaveform {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pulse_index < self.payload.total_bits() * 2 {
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

impl Source for DataWaveform {
    fn channels(&self) -> ChannelCount { 1 }
    fn sample_rate(&self) -> SampleRate { self.config.sample_rate }
    fn current_span_len(&self) -> Option<usize> { None }

    fn total_duration(&self) -> Option<Duration> {
        let mut duration = Duration::ZERO;
        let mut pulse = self.current_pulse.clone();
        let bit_counts = self.payload.bit_counts();
        pulse.length = self.length_pulse_one;
        duration += pulse.duration() * bit_counts.ones as u32 * 2;
        pulse.length = self.length_pulse_zero;
        duration += pulse.duration() * bit_counts.zeros as u32 * 2;
        return Some(duration);
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        if self.payload.len() == 0 { return Ok(()) }
        let samples = (pos.as_secs_f64() * self.config.sample_rate as f64).round() as usize;

        // Use the pos / total_duration percentage time progress
        // to estimate the byte position within the payload.
        let estimated_byte_index = std::cmp::min((self.payload.len() as f32 * pos.div_duration_f32(self.total_duration().unwrap())) as usize, self.payload.len() - 1);

        // Calculate the number of samples in the start to estimated byte position range.
        let mut pulse = self.current_pulse.clone();
        let mut pulse_samples: usize = 0;
        let bit_counts_to_index = self.payload.bit_counts_for_range(0..estimated_byte_index).unwrap();
        pulse.length = self.length_pulse_one;
        pulse_samples += pulse.len() as usize * bit_counts_to_index.ones * 2;
        pulse.length = self.length_pulse_zero;
        pulse_samples += pulse.len() as usize * bit_counts_to_index.zeros * 2;

        self.current_pulse_index = estimated_byte_index * 16;
        self.current_pulse_sample_index = 0;

        while pulse_samples > samples && self.current_pulse_index > 0 {
            pulse_samples -= self.current_pulse.len() as usize;
            self.current_pulse_index -= 1;
            self.update_pulse();
        }

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

impl Waveform for DataWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn started(&self) -> bool { self.current_pulse_index > 0 || self.current_pulse_sample_index > 0 }

    fn visualise(&self, pulse_string_length: usize) -> String {
        let mut pulse_string = "".to_string();
        let mut pulse_index = self.current_pulse_index;
        let mut current_high = self.current_pulse.high;
        let mut current_char: char;
        while pulse_string.chars().count() < pulse_string_length && pulse_index < self.payload.total_bits() * 2 {
            current_char = if current_high { '\u{2588}' } else { ' ' };
            pulse_string.push(current_char);
            if self.payload.data.view_bits::<Msb0>()[pulse_index as usize / 2] && pulse_string.chars().count() < pulse_string_length {
                pulse_string.push(current_char);
            }
            pulse_index += 1;
            current_high = !current_high;
        }
        return pulse_string;
    }

    fn payload_with_position(&self) -> Option<DataPayloadWithPosition> {
        Some(
            DataPayloadWithPosition {
                payload: self.payload.clone(),
                current_byte_index: self.current_pulse_index as usize / 16,
                current_bit_index: ((self.current_pulse_index / 2) % 8 ) as u8,
            }
        )
    }
}
