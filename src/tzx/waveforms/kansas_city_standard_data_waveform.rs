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
    blocks::kansas_city_standard_data_block::KCSBitByteConfig,
    data::{DataPayload, DataPayloadWithPosition},
    waveforms::{Pulse, Waveform}
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum KCSBitState {
    #[default]
    Start,
    Data,
    Stop,
}

/// Iterates pulses for use by a [DataWaveform].
#[derive(Clone, Debug)]
pub struct KCSDataPulseIterator {
    length_pulse_zero: u16,
    length_pulse_one: u16,
    bit_byte_config: KCSBitByteConfig,
    payload: DataPayload,
    start_pulse_high: bool,
    current_pulse: Pulse,
    bit_state: KCSBitState,
    bit_pulse_index: u8,
    bit_index: u8,
    payload_byte_index: usize,
}

impl KCSDataPulseIterator {
    pub fn new(
        config: Arc<Config>,
        length_pulse_zero: u16,
        length_pulse_one: u16,
        bit_byte_config: KCSBitByteConfig,
        payload: DataPayload,
        start_pulse_high: bool
    ) -> Self {
        return Self {
            length_pulse_zero,
            length_pulse_one,
            bit_byte_config,
            payload,
            start_pulse_high,
            current_pulse: Pulse::new(config, 0, start_pulse_high),
            bit_state: KCSBitState::Start,
            bit_pulse_index: 0,
            bit_index: 0,
            payload_byte_index: 0,
        }
    }
}

impl Iterator for KCSDataPulseIterator {
    type Item = Pulse;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_pulses_remaining() { return None }

        // // First pulse.
        // if !self.started {
        //     self.started = true;
        //     return Some(self.current_pulse.clone());
        // }

        // If we're in start state and there's no leading bits, move directly to data state.
        if self.bit_pulse_index == 0 && self.bit_state == KCSBitState::Start && self.bit_byte_config.count_pulses_leading() == 0 {
            self.bit_state = KCSBitState::Data;
        }

        let current_bit = match self.bit_state {
            KCSBitState::Start => self.bit_byte_config.value_leading_bits().value() == 1,
            KCSBitState::Data => match self.bit_byte_config.endianness().value() == 1 {
                true => self.payload.data[self.payload_byte_index].view_bits::<Msb0>()[self.bit_index as usize],
                false => self.payload.data[self.payload_byte_index].view_bits::<Lsb0>()[self.bit_index as usize],
            },
            KCSBitState::Stop => self.bit_byte_config.value_trailing_bits().value() == 1,
        };
        // eprintln!("Byte: {}", self.payload.data[self.payload_byte_index]);
        // eprintln!("{:?}|{:?}|{:?}|{:?}|{:?}|{:?}", current_bit, self.current_pulse_index, self.bit_state, self.bit_pulse_index, self.bit_index, self.payload_byte_index);
        self.current_pulse.length = if current_bit { self.length_pulse_one } else { self.length_pulse_zero };
        self.current_pulse.high = !((self.bit_pulse_index % 2 == 0) ^ self.start_pulse_high);

        // Advance pulse within bit.
        self.bit_pulse_index += 1;
        if self.bit_pulse_index == match self.bit_state {
            KCSBitState::Start => self.bit_byte_config.count_pulses_leading(),
            KCSBitState::Data => if current_bit { self.bit_byte_config.count_pulses_one() } else { self.bit_byte_config.count_pulses_zero() },
            KCSBitState::Stop => self.bit_byte_config.count_pulses_trailing(),
        } {
            self.bit_pulse_index = 0;

            // Advance bit.
            self.bit_index += 1;
            if self.bit_index == match self.bit_state {
                KCSBitState::Start => self.bit_byte_config.count_leading_bits().value(),
                KCSBitState::Data => 8,
                KCSBitState::Stop => self.bit_byte_config.count_trailing_bits().value()
            } {
                self.bit_index = 0;
                match self.bit_state {
                    KCSBitState::Start => self.bit_state = KCSBitState::Data,
                    KCSBitState::Data => if self.bit_byte_config.count_pulses_trailing() == 0 {
                        self.bit_state = KCSBitState::Start;
                        self.payload_byte_index += 1;
                    } else {
                        self.bit_state = KCSBitState::Stop
                    }
                    KCSBitState::Stop => {
                        self.bit_state = KCSBitState::Start;
                        self.payload_byte_index += 1;
                    }
                }
            }
        }

        Some(self.current_pulse.clone())
    }
}

impl KCSDataPulseIterator {
    fn len(&self) -> usize {
        let bit_counts = self.payload.bit_counts();

        self.payload.len() * self.bit_byte_config.start_stop_pulses_per_byte()
        + bit_counts.zeros * self.bit_byte_config.count_pulses_zero() as usize
        + bit_counts.ones * self.bit_byte_config.count_pulses_one() as usize
    }

    fn has_pulses_remaining(&self) -> bool { self.payload_byte_index < self.payload.len() }

    fn current_pulse_index(&self) -> usize {
        if self.payload_byte_index >= self.payload.len() { return self.len() }

        let bit_counts = self.payload.bit_counts_for_range(0..self.payload_byte_index).unwrap();

        self.payload_byte_index * self.bit_byte_config.start_stop_pulses_per_byte()
        + bit_counts.zeros * self.bit_byte_config.count_pulses_zero() as usize
        + bit_counts.ones * self.bit_byte_config.count_pulses_one() as usize
        + match self.bit_state {
            KCSBitState::Start => self.bit_index * self.bit_byte_config.count_pulses_leading() + self.bit_pulse_index,
            KCSBitState::Data => {
                let mut byte = self.payload.data[self.payload_byte_index].clone();
                let bits = byte.view_bits_mut::<Lsb0>();
                if self.bit_byte_config.is_msb() {
                    bits.reverse();
                }

                bits[..self.bit_index as usize]
                    .iter()
                    .map(|b| self.bit_byte_config.count_pulses_bit(*b))
                    .reduce(|acc, cpb| acc + cpb).unwrap_or_default()
            },
            KCSBitState::Stop => self.bit_index * self.bit_byte_config.count_pulses_trailing() + self.bit_pulse_index,
        } as usize
        + self.bit_pulse_index as usize
    }

    pub fn started(&self) -> bool { self.bit_pulse_index > 0 || self.bit_index > 0 || self.payload_byte_index > 0 }

    pub fn payload_with_position(&self) -> DataPayloadWithPosition {
        DataPayloadWithPosition {
            payload: self.payload.clone(),
            current_byte_index: self.payload_byte_index,
            current_bit_index: match self.bit_byte_config.endianness().value() {
                // MSB
                1 => match self.bit_state {
                    KCSBitState::Start => 0,
                    KCSBitState::Data => self.bit_index,
                    KCSBitState::Stop => 7,
                }
                // LSB
                _ => match self.bit_state {
                    KCSBitState::Start => 7,
                    KCSBitState::Data => 8 - self.bit_index,
                    KCSBitState::Stop => 0,
                }
            }
        }

    }
}

impl From<KCSDataPulseIterator> for DataPayloadWithPosition {
    fn from(pulse_iterator: KCSDataPulseIterator) -> Self { pulse_iterator.payload_with_position() }
}

/// A waveform for standard data encoding as used by [StandardSpeedDataBlock](crate::tzx::blocks::StandardSpeedDataBlock),
/// [TurboSpeedDataBlock](crate::tzx::blocks::TurboSpeedDataBlock), and [PureDataBlock](crate::tzx::blocks::PureDataBlock).
///
/// Data is encoded as a sequence of pulse pairs, each pair consisting of one high and one low pulse to form a complete wave.
/// The length of the pulses in each pair determines whether the pulse pair encodes a one or a zero. The data payload is encoded
/// linearly by most significant bit. The pulse sequence is generated by [DataPulseIterator], and the waveform itself iterates
/// the samples.
#[derive(Clone, Debug)]
pub struct KansasCityStandardDataWaveform {
    config: Arc<Config>,
    length_pulse_zero: u16,
    length_pulse_one: u16,
    payload: DataPayload,
    pulse_iterator: KCSDataPulseIterator,
    current_pulse: Pulse,
    current_pulse_sample_index: usize,
    cached_symbol_pulse_lengths: OnceLock<HashMap<u16,u16>>,
}

impl KansasCityStandardDataWaveform {
    // Constructs a Kansas City Standard data waveform.
    pub fn new(
        config: Arc<Config>,
        length_pulse_zero: u16,
        length_pulse_one: u16,
        bit_byte_config: KCSBitByteConfig,
        payload: DataPayload,
        start_pulse_high: bool
    ) -> Self {
        let mut pulse_iterator = KCSDataPulseIterator::new(
            config.clone(),
            length_pulse_zero,
            length_pulse_one,
            bit_byte_config.clone(),
            payload.clone(),
            start_pulse_high
        );
        let current_pulse = pulse_iterator.next().unwrap_or(Pulse::new(config.clone(), 0, false));

        return Self {
            config,
            length_pulse_zero,
            length_pulse_one,
            payload,
            pulse_iterator,
            current_pulse,
            current_pulse_sample_index: 0,
            cached_symbol_pulse_lengths: OnceLock::new(),
        }
    }

    fn compute_symbol_pulse_lengths(&self) -> HashMap<u16, u16> {
        let mut symbol_pulse_lengths = HashMap::new();
        let mut shortest: u16 = self.length_pulse_one;

        // Collect pulse lengths from all symbols and determine shortest length.
        symbol_pulse_lengths.entry(self.length_pulse_one).or_insert(0 as u16);
        symbol_pulse_lengths.entry(self.length_pulse_zero).or_insert(0 as u16);
        if self.length_pulse_zero < shortest { shortest = self.length_pulse_zero }

        // Compute the integer ratios of each length to the shortest length to
        // facilitate visualisation.
        for (length, ratio) in symbol_pulse_lengths.iter_mut() {
            *ratio = length / shortest;
        }

        return symbol_pulse_lengths;
    }

    fn samples_duration_to_byte_index(&self, byte_index: usize) -> (usize, Duration) {
        let mut count_ones: usize;
        let mut count_zeros: usize;
        let mut samples: usize = 0;
        let mut duration = Duration::ZERO;
        let mut pulse = self.current_pulse.clone();
        let bit_counts = match byte_index >= self.payload.len() {
            true => *self.payload.bit_counts(),
            false => self.payload.bit_counts_for_range(0..byte_index).unwrap(),
        };
        let byte_index = if byte_index > self.payload.len() { self.payload.len() } else { byte_index };

        count_ones = bit_counts.ones; //* self.pulse_iterator.bit_byte_config.count_pulses_one() as usize;
        count_zeros = bit_counts.zeros; //* self.pulse_iterator.bit_byte_config.count_pulses_zero() as usize;

        match self.pulse_iterator.bit_byte_config.value_leading_bits().value() {
            0 => count_zeros += self.pulse_iterator.bit_byte_config.count_leading_bits().value() as usize * byte_index,
            1 => count_ones += self.pulse_iterator.bit_byte_config.count_leading_bits().value() as usize * byte_index,
            _ => panic!("u1 invalid value"),
        }

        match self.pulse_iterator.bit_byte_config.value_trailing_bits().value() {
            0 => count_zeros += self.pulse_iterator.bit_byte_config.count_trailing_bits().value() as usize * byte_index,
            1 => count_ones += self.pulse_iterator.bit_byte_config.count_trailing_bits().value() as usize * byte_index,
            _ => panic!("u1 invalid value"),
        }

        let count_pulses_ones = count_ones * self.pulse_iterator.bit_byte_config.count_pulses_one() as usize;
        let count_pulses_zeros = count_zeros * self.pulse_iterator.bit_byte_config.count_pulses_zero() as usize;

        pulse.length = self.length_pulse_one;
        samples += pulse.len() as usize * count_pulses_ones as usize;
        duration += pulse.duration() * count_pulses_ones as u32;

        pulse.length = self.length_pulse_zero;
        samples += pulse.len() as usize * count_pulses_zeros as usize;
        duration += pulse.duration() * count_pulses_zeros as u32;

        return (samples, duration);
    }
}

impl fmt::Display for KansasCityStandardDataWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bit_counts = self.payload.bit_counts();
        write!(f, "KansasCityStandardDataWaveform:   {:6} / {:6} pulses (0+1: {}+{}; w0: {:.2}→{}; w1: {:.2}→{})",
            self.pulse_iterator.current_pulse_index(),
            self.pulse_iterator.len(),
            bit_counts.zeros,
            bit_counts.ones,
            self.length_pulse_zero as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64,
            (self.length_pulse_zero as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64).round() as u32,
            self.length_pulse_one as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64,
            (self.length_pulse_one as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64).round() as u32,
        )
    }
}

impl Iterator for KansasCityStandardDataWaveform {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let pulse_sample: Option<Self::Item> = self.current_pulse.next_sample(self.current_pulse_sample_index as u32);
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

impl Source for KansasCityStandardDataWaveform {
    fn channels(&self) -> ChannelCount { 1 }
    fn sample_rate(&self) -> SampleRate { self.config.sample_rate }
    fn current_span_len(&self) -> Option<usize> { None }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.samples_duration_to_byte_index(self.payload.len()).1)
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        if self.payload.len() == 0 { return Ok(()) }
        let samples = (pos.as_secs_f64() * self.config.sample_rate as f64).round() as usize;

        // Use the pos / total_duration percentage time progress
        // to estimate the byte position within the payload.
        let estimated_byte_index = std::cmp::min((self.payload.len() as f32 * pos.div_duration_f32(self.total_duration().unwrap())) as usize, self.payload.len() - 1);
        let estimated_byte_index = if estimated_byte_index > 0 { estimated_byte_index - 1 } else { 0 };

        // Calculate number of samples to estimated_byte_index
        let (mut seek_samples, _) = self.samples_duration_to_byte_index(estimated_byte_index);
        // eprintln!("{:?}/{:?} : {:?}/{:?}", estimated_byte_index, self.payload.len(), seek_samples, samples);

        // Set pulse iterator to the estimated byte_index and reset to start of byte.
        self.pulse_iterator.payload_byte_index = estimated_byte_index;
        self.pulse_iterator.bit_index = 0;
        self.pulse_iterator.bit_pulse_index = 0;
        self.pulse_iterator.bit_state = KCSBitState::Start;

        self.current_pulse_sample_index = 0;

        // while pulse_samples > samples && let Some(prev_pulse) = self.pulse_iterator.prev() {
        //     self.current_pulse = prev_pulse;
        //     pulse_samples -= self.current_pulse.len() as usize;
        // }

        while seek_samples < samples {
            seek_samples += self.current_pulse.len() as usize;
            if seek_samples > samples {
                self.current_pulse_sample_index = seek_samples - samples;
            } else {
                if let Some(next_pulse) = self.pulse_iterator.next() {
                    self.current_pulse = next_pulse;
                }
            }
        }
        return Ok(());
    }
}

impl Waveform for KansasCityStandardDataWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }

    fn started(&self) -> bool { self.pulse_iterator.started() || self.current_pulse_sample_index > 0 }

    fn current_baud(&self) -> Option<usize> {
        let target_duration = Duration::from_millis(100);
        let mut duration = Duration::ZERO;
        let mut pulses: usize = 0;
        let mut pulse_iterator = self.pulse_iterator.clone();

        while duration < target_duration {
            if let Some(pulse) = pulse_iterator.next() {
                duration += pulse.duration();
                pulses += 1;
            } else {
                break;
            }
        }

        let avg_pulses_per_byte = self.pulse_iterator.bit_byte_config.start_stop_pulses_per_byte()
            + self.pulse_iterator.bit_byte_config.count_pulses_one() as usize * 4
            + self.pulse_iterator.bit_byte_config.count_pulses_zero() as usize * 4;
        let bits_per_byte = 8
            + self.pulse_iterator.bit_byte_config.count_leading_bits().value()
            + self.pulse_iterator.bit_byte_config.count_trailing_bits().value();

        Some(((pulses * bits_per_byte as usize) as f64 / (duration.as_secs_f64() * avg_pulses_per_byte as f64)).round() as usize)
        // Some((pulses as f64 / (duration.as_secs_f64() * 2)).round() as usize)
    }

    fn visualise(&self, pulse_string_length: usize) -> String {
        let symbol_pulse_lengths = self.cached_symbol_pulse_lengths.get_or_init(|| self.compute_symbol_pulse_lengths());

        let mut pulse_string = "".to_string();
        let mut pulse_iterator = self.pulse_iterator.clone();
        let mut pulse = self.current_pulse.clone();
        let mut current_char: char;
        let mut chars_to_print: u16;

        while pulse_string.chars().count() < pulse_string_length && pulse_iterator.has_pulses_remaining() {
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

    fn payload_with_position(&self) -> Option<DataPayloadWithPosition> {
        Some(DataPayloadWithPosition::from(self.pulse_iterator.clone()))
    }
}
