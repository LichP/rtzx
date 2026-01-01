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

#[derive(Clone)]
pub struct DataPulseIterator {
    length_pulse_zero: u16,
    length_pulse_one: u16,
    payload: DataPayload,
    start_pulse_high: bool,
    current_pulse: Pulse,
    current_pulse_index: usize,
}

impl DataPulseIterator {
    pub fn new(config: Arc<Config>, length_pulse_zero: u16, length_pulse_one: u16, payload: DataPayload, start_pulse_high: bool) -> Self {
        return Self {
            length_pulse_zero,
            length_pulse_one,
            payload,
            start_pulse_high,
            current_pulse: Pulse::new(config, 0, false),
            current_pulse_index: 0,
        }
    }
}

impl Iterator for DataPulseIterator {
    type Item = Pulse;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pulse_index < self.len() {
            let current_bit = self.payload.data.view_bits::<Msb0>()[self.current_pulse_index as usize / 2];
            self.current_pulse.length = if current_bit { self.length_pulse_one } else { self.length_pulse_zero };
            self.current_pulse.high = !((self.current_pulse_index % 2 == 0) ^ self.start_pulse_high);
            self.current_pulse_index += 1;
            Some(self.current_pulse.clone())
        } else {
            None
        }
    }
}

impl DataPulseIterator {
    fn len(&self) -> usize { self.payload.total_bits() * 2 }

    fn prev(&mut self) -> Option<Pulse> {
        if self.current_pulse_index > 0 {
            self.current_pulse_index -= 1;
            let current_bit = self.payload.data.view_bits::<Msb0>()[self.current_pulse_index as usize / 2];
            self.current_pulse.length = if current_bit { self.length_pulse_one } else { self.length_pulse_zero };
            self.current_pulse.high = !((self.current_pulse_index % 2 == 0) ^ self.start_pulse_high);
            Some(self.current_pulse.clone())
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct DataWaveform {
    config: Arc<Config>,
    length_pulse_zero: u16,
    length_pulse_one: u16,
    payload: DataPayload,
    pulse_iterator: DataPulseIterator,
    current_pulse: Pulse,
    current_pulse_sample_index: usize,
}

impl DataWaveform {
    pub fn new(config: Arc<Config>, length_pulse_zero: u16, length_pulse_one: u16, payload: DataPayload, start_pulse_high: bool) -> Self {
        let mut pulse_iterator = DataPulseIterator::new(config.clone(), length_pulse_zero, length_pulse_one, payload.clone(), start_pulse_high);
        let current_pulse = pulse_iterator.next().unwrap_or(Pulse::new(config.clone(), 0, false));

        return Self {
            config,
            length_pulse_zero,
            length_pulse_one,
            payload,
            pulse_iterator,
            current_pulse,
            current_pulse_sample_index: 0,
        }
    }
}

impl fmt::Display for DataWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bit_counts = self.payload.bit_counts();
        write!(f, "DataWaveform:   {:6} / {:6} pulses (0+1: {}+{}; w0: {:.2}→{}; w1: {:.2}→{})",
            self.pulse_iterator.current_pulse_index,
            bit_counts.total * 2,
            bit_counts.zeros,
            bit_counts.ones,
            self.length_pulse_zero as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64,
            (self.length_pulse_zero as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64).round() as u32,
            self.length_pulse_one as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64,
            (self.length_pulse_one as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64).round() as u32,
        )
    }
}

impl Iterator for DataWaveform {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let pulse_sample: Option<Self::Item> = self.current_pulse.get_next_sample(self.current_pulse_sample_index as u32);
        if pulse_sample.is_some() {
            self.current_pulse_sample_index += 1;
            return pulse_sample;
        }
        let next_pulse = self.pulse_iterator.next();
        if next_pulse.is_some() {
            self.current_pulse = next_pulse.unwrap();
            self.current_pulse_sample_index = 0;
            return self.next();
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

        self.pulse_iterator.current_pulse_index = estimated_byte_index * 16;
        self.current_pulse_sample_index = 0;

        while pulse_samples > samples && let Some(prev_pulse) = self.pulse_iterator.prev() {
            self.current_pulse = prev_pulse;
            pulse_samples -= self.current_pulse.len() as usize;
        }

        while pulse_samples < samples {
            pulse_samples += self.current_pulse.len() as usize;
            if pulse_samples > samples {
                self.current_pulse_sample_index = pulse_samples - samples;
            } else {
                if let Some(next_pulse) = self.pulse_iterator.next() {
                    self.current_pulse = next_pulse;
                }
            }
        }
        return Ok(());
    }
}

impl Waveform for DataWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn started(&self) -> bool { self.pulse_iterator.current_pulse_index > 0 || self.current_pulse_sample_index > 0 }

    fn visualise(&self, pulse_string_length: usize) -> String {
        let mut pulse_string = "".to_string();

        let mut pulse_index = self.pulse_iterator.current_pulse_index;
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
                current_byte_index: self.pulse_iterator.current_pulse_index as usize / 16,
                current_bit_index: ((self.pulse_iterator.current_pulse_index / 2) % 8 ) as u8,
            }
        )
    }
}
