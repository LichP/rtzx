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
pub struct DataWaveform {
    config: Arc<Config>,
    length_pulse_zero: u16,
    length_pulse_one: u16,
    data: Vec<u8>,
    used_bits: u8,
    current_pulse_index: usize,
    count_set_bits: u64,
    pulses: Vec<Pulse>,
}

impl DataWaveform {
    pub fn new(config: Arc<Config>, length_pulse_zero: u16, length_pulse_one: u16, data: &Vec<u8>, used_bits: u8, start_pulse_high: bool) -> Self {
        let mut pulses: Vec<Pulse> = vec![];

        for (index, byte) in data.iter().enumerate() {
            let mut data_bit_index: u8 = 0;
            while data_bit_index < 8 {
                if index == data.len() - 1 && data_bit_index + 1 > used_bits {
                    data_bit_index += 1;
                    continue;
                }
                let bit_mask: u8 = 1 << (7 - data_bit_index);
                let current_bit_set = byte & bit_mask == bit_mask;
                let length_pulse = if current_bit_set { length_pulse_one } else { length_pulse_zero };
                pulses.push(Pulse::new(config.clone(), length_pulse, start_pulse_high));
                pulses.push(Pulse::new(config.clone(), length_pulse,!start_pulse_high));
                data_bit_index += 1;
            }
        }

        return Self {
            config,
            length_pulse_zero,
            length_pulse_one,
            data: data.to_owned(),
            used_bits,
            current_pulse_index: 0,
            count_set_bits: popcnt::count_ones(data.as_slice()),
            pulses: pulses,
        }
    }
}

impl fmt::Display for DataWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DataWaveform:   {:6} / {:6} pulses",
            self.current_pulse_index,
            self.pulses.len(),
        )
    }
}

impl Iterator for DataWaveform {
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

impl Source for DataWaveform {
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

impl Waveform for DataWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn visualise(&self) -> String {
        let pulse_string_length = 32;
        let mut pulse_string = "".to_string();
        let mut pulse_index = self.current_pulse_index;
        let mut current_char: char;
        while pulse_string.chars().count() < pulse_string_length && pulse_index < self.pulses.len() {
            current_char = if self.pulses[pulse_index].high { '\u{2588}' } else { ' ' };
            pulse_string.push(current_char);
            if self.pulses[pulse_index].length == self.length_pulse_one && pulse_string.chars().count() < pulse_string_length {
                pulse_string.push(current_char);
            }
            pulse_index += 1;
        }
        return pulse_string;
    }
}
