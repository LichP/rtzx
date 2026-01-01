use bitvec::prelude::*;
use rodio::{
    ChannelCount,
    SampleRate,
    Source,
    source::SeekError,
};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use crate::tzx::{
    Config,
    blocks::generalized_data_block::SymbolDefinition,
    data::DataPayload,
    waveforms::{Pulse, Waveform},
};

#[derive(Clone)]
pub struct GeneralizedPulseIterator {
    symbols: Arc<Vec<SymbolDefinition>>,
    payload: DataPayload,
    current_payload_bit_index: usize,
    current_symbol_key: u8,
    current_pulse: Pulse,
    current_pulse_index: usize,
    started: bool,
}

impl GeneralizedPulseIterator {
    pub fn new(config: Arc<Config>, symbols: Arc<Vec<SymbolDefinition>>, payload: DataPayload, start_pulse_high: bool) -> Self {
        let range= 0..symbols.len().ilog2() as usize;
        let current_symbol_key = payload.data.view_bits::<Msb0>()[range].load_be::<u8>();

        return Self {
            symbols: symbols.clone(),
            payload,
            current_payload_bit_index: 0,
            current_symbol_key,
            current_pulse: Pulse::new(config, symbols[current_symbol_key as usize].pulses[0], start_pulse_high),
            current_pulse_index: 0,
            started: false,
         }
    }

    fn current_symbol(&self) -> &SymbolDefinition { &self.symbols[self.current_symbol_key as usize] }

    fn symbol_key_size(&self) -> usize { self.symbols.len().ilog2() as usize }

    fn next_symbol_key(&mut self) -> Option<u8> {
        self.current_payload_bit_index += self.symbol_key_size();
        if self.has_data_remaining() {
            let range= self.current_payload_bit_index..(self.current_payload_bit_index + self.symbol_key_size());
            Some(self.payload.data.view_bits::<Msb0>()[range].load_be::<u8>())
        } else {
            None
        }
    }

    fn has_data_remaining(&self) -> bool { self.current_payload_bit_index + self.symbol_key_size() - 1 < self.payload.total_bits()  }

    fn update_pulse(&mut self) {
        self.current_pulse.length = self.current_symbol().pulses[self.current_pulse_index];
        self.current_pulse.high = if self.current_pulse_index == 0 {
            self.current_symbol().polarity.next_polarity(self.current_pulse.high)
        } else {
            !self.current_pulse.high
        }
    }

    fn seek_and_fetch(&mut self, bit_index: usize) -> Option<Pulse> {
        self.current_payload_bit_index = bit_index - (bit_index % self.symbol_key_size());
        if self.has_data_remaining() {
            let range= self.current_payload_bit_index..(self.current_payload_bit_index + self.symbol_key_size());
            self.current_symbol_key = self.payload.data.view_bits::<Msb0>()[range].load_be::<u8>();
            self.update_pulse();
            return Some(self.current_pulse.clone());
        } else {
            None
        }
    }
}

impl Iterator for GeneralizedPulseIterator {
    type Item = Pulse;

    fn next(&mut self) -> Option<Self::Item> {
        // First pulse.
        if !self.started {
            self.started = true;
            return Some(self.current_pulse.clone());
        }

        self.current_pulse_index += 1;
        if self.current_pulse_index < self.current_symbol().pulses.len() && self.current_symbol().pulses[self.current_pulse_index] != 0 {
            self.update_pulse();
            return Some(self.current_pulse.clone());
        } else {
            if let Some(next_symbol_key) = self.next_symbol_key() {
                self.current_pulse_index = 0;
                self.current_symbol_key = next_symbol_key;
                self.update_pulse();
                return Some(self.current_pulse.clone());
            }
        }

        return None;
    }
}

pub struct GeneralizedWaveform {
    config: Arc<Config>,
    symbols: Arc<Vec<SymbolDefinition>>,
    payload: DataPayload,
    start_pulse_high: bool,
    last_pulse_high: bool,
    pulse_iterator: GeneralizedPulseIterator,
    current_pulse: Pulse,
    current_pulse_sample_index: usize,
    cached_seek_data: (Duration, usize, usize),
    cached_symbol_pulse_lengths: OnceLock<HashMap<u16,u16>>,
    cached_total_duration: Duration,
    cached_total_pulses: usize,
}

impl Clone for GeneralizedWaveform {
    fn clone(&self) -> Self {
        GeneralizedWaveform::new(
            self.config.clone(),
            self.symbols.clone(),
            self.payload.clone(),
            self.start_pulse_high.clone(),
        )
    }
}

impl GeneralizedWaveform {
    pub fn new(config: Arc<Config>, symbols: Arc<Vec<SymbolDefinition>>, payload: DataPayload, start_pulse_high: bool) -> Self {
        let mut pulse_iterator = GeneralizedPulseIterator::new(config.clone(), symbols.clone(), payload.clone(), start_pulse_high);
        let pulse_iterator_for_totals = pulse_iterator.clone();

        let mut total_duration = Duration::ZERO;
        let mut total_pulses = 0;
        let mut last_pulse_high = false;

        for pulse in pulse_iterator_for_totals {
            total_duration += pulse.duration();
            total_pulses += 1;
            last_pulse_high = pulse.high;
        }

        let current_pulse = pulse_iterator.next().unwrap_or(Pulse::new(config.clone(), 0, false));

        return Self {
            config,
            symbols,
            payload: payload.clone(),
            start_pulse_high,
            last_pulse_high,
            pulse_iterator,
            current_pulse,
            current_pulse_sample_index: 0,
            cached_seek_data: (Duration::ZERO, 0, 0),
            cached_symbol_pulse_lengths: OnceLock::new(),
            cached_total_duration: total_duration,
            cached_total_pulses: total_pulses,
        };
    }

    fn compute_symbol_pulse_lengths(&self) -> HashMap<u16, u16> {
        let mut symbol_pulse_lengths = HashMap::new();
        let mut shortest: u16 = 65535;

        // Collect pulse lengths from all symbols and determine shortest length.
        for symbol in self.symbols.as_slice() {
            for &length in &symbol.pulses {
                if length > 0 {
                    symbol_pulse_lengths.entry(length).or_insert(0 as u16);
                    if length < shortest { shortest = length }
                }
            }
        }

        // Compute the integer ratios of each length to the shortest length to
        // facilitate visualisation.
        for (length, ratio) in symbol_pulse_lengths.iter_mut() {
            *ratio = length / shortest;
        }

        return symbol_pulse_lengths;
    }

    pub fn last_pulse_high(&self) -> bool { self.last_pulse_high }
}

impl fmt::Display for GeneralizedWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GeneralizedWaveform:   {:6} / {:6} symbols ({} pulses)",
            self.pulse_iterator.current_payload_bit_index / self.pulse_iterator.symbol_key_size(),
            self.payload.total_bits() / self.pulse_iterator.symbol_key_size(),
            self.cached_total_pulses,
        )
    }
}

impl Iterator for GeneralizedWaveform {
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

impl Source for GeneralizedWaveform {
    fn channels(&self) -> ChannelCount { 1 }
    fn sample_rate(&self) -> SampleRate { self.config.sample_rate }
    fn current_span_len(&self) -> Option<usize> { None }

    fn total_duration(&self) -> Option<Duration> { Some(self.cached_total_duration) }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        if self.payload.len() == 0 { return Ok(()) }
        let samples = (pos.as_secs_f64() * self.config.sample_rate as f64).round() as usize;

        // Use cached seek data to speed up seek if we're seeking forwards.
        let (cached_pos, mut pulse_samples, mut payload_bit_index) = self.cached_seek_data;
        if pos < cached_pos {
            pulse_samples = 0;
            payload_bit_index = 0;
        }

        self.pulse_iterator.current_payload_bit_index = payload_bit_index;
        self.current_pulse_sample_index = 0;
        self.current_pulse = self.pulse_iterator.seek_and_fetch(payload_bit_index).unwrap();

        while pulse_samples < samples && self.pulse_iterator.has_data_remaining() {
            pulse_samples += self.current_pulse.len() as usize;
            if pulse_samples > samples {
                self.current_pulse_sample_index = pulse_samples - samples;
            } else {
                self.current_pulse = self.pulse_iterator.next().unwrap();
            }
        }

        self.cached_seek_data = (pos, pulse_samples, self.pulse_iterator.current_payload_bit_index);

        return Ok(());
    }
}

impl Waveform for GeneralizedWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn started(&self) -> bool { self.pulse_iterator.current_pulse_index > 0 || self.current_pulse_sample_index > 0 }

    fn visualise(&self, pulse_string_length: usize) -> String {
        let symbol_pulse_lengths = self.cached_symbol_pulse_lengths.get_or_init(|| self.compute_symbol_pulse_lengths());

        let mut pulse_string = "".to_string();
        let mut pulse_iterator = self.pulse_iterator.clone();
        let mut pulse = self.current_pulse.clone();
        let mut current_char: char;
        let mut chars_to_print: u16;

        while pulse_string.chars().count() < pulse_string_length && pulse_iterator.has_data_remaining() {
            current_char = if pulse.high { '\u{2588}' } else { ' ' };
            chars_to_print = *symbol_pulse_lengths.get(&pulse.length).unwrap_or(&(1 as u16));

            for _ in 0..chars_to_print {
                if pulse_string.chars().count() < pulse_string_length {
                    pulse_string.push(current_char);
                }
            }

            pulse = pulse_iterator.next().unwrap();
        }

        return pulse_string;
    }
}
