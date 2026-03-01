use bilge::prelude::*;
use binrw::{
    binrw,
};
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::tzx::{
    Config,
    ExtendedDisplayCollector,
    blocks::{Block, BlockType},
    data::DataPayload,
    waveforms::{
        KansasCityStandardDataWaveform,
        PauseType,
        PauseWaveform,
        PilotWaveform,
        Waveform,
    },
};

#[bitsize(16)]
#[derive(Clone, Copy, DebugBits, FromBits)]
pub struct KCSBitByteConfig {
    count_pulses_one_u4: u4,      // Number of ONE pulses in a ONE bit (0=16 pulese) {8}
    count_pulses_zero_u4: u4,     // Number of ZERO pulses in a ZERO bit (0=16 pulses) {4}
    pub endianness: u1,           // Endianness (0 for LSb first, 1 for MSb first) {0}
    bit_reserved: u1,             // Reserved
    pub value_trailing_bits: u1,  // Value of trailing bits {1}
    pub count_trailing_bits: u2,  // Number of trailing bits {2}
    pub value_leading_bits: u1,   // Value of leading bits {0}
    pub count_leading_bits: u2,   // Number of leading bits {1}
}

impl Default for KCSBitByteConfig {
    fn default() -> Self {
        Self::new(
            u4::new(8),
            u4::new(4),
            u1::new(0),
            u1::new(0),
            u1::new(1),
            u2::new(2),
            u1::new(0),
            u2::new(1),
        )
    }
}

impl KCSBitByteConfig {
    pub fn count_pulses_zero(self) -> u8 {
        // Odd numbers of pulses per bit are not supported: we only expect to get even numbers.
        match self.count_pulses_zero_u4().value() {
            0 => 16,
            odd if odd % 2 == 1 => odd - 1,
            even => even,
        }
    }

    pub fn count_pulses_one(self) -> u8 {
        // Odd numbers of pulses per bit are not supported: we only expect to get even numbers.
        match self.count_pulses_one_u4().value() {
            0 => 16,
            odd if odd % 2 == 1 => odd - 1,
            even => even,
        }
    }

    pub fn count_pulses_bit(self, bit: bool) -> u8 {
        match bit {
            true => self.count_pulses_one(),
            false => self.count_pulses_zero(),
        }
    }

    pub fn start_stop_pulses_per_byte(self) -> usize {
        self.count_leading_bits().value() as usize * self.count_pulses_leading() as usize
        +
        self.count_trailing_bits().value() as usize * self.count_pulses_trailing() as usize
    }

    pub fn count_pulses_leading(self) -> u8 {
        match self.value_leading_bits().value() {
            0 => self.count_pulses_zero(),
            1 => self.count_pulses_one(),
            _ => panic!("u1 invalid value"),
        }
    }

    pub fn count_pulses_trailing(self) -> u8 {
        match self.value_trailing_bits().value() {
            0 => self.count_pulses_zero(),
            1 => self.count_pulses_one(),
            _ => panic!("u1 invalid value"),
        }
    }

    pub fn is_msb(self) -> bool { self.endianness().value() == 1 }
}

impl fmt::Display for KCSBitByteConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "p:0={}/1={};l:{}x{};t:{}x{};{}",
            self.count_pulses_zero(),
            self.count_pulses_one(),
            self.value_leading_bits(),
            self.count_leading_bits(),
            self.value_trailing_bits(),
            self.count_trailing_bits(),
            if self.endianness().into() { "msb" } else { "lsb" }
        )
    }
}

/// A [A Kansas City Standard Data Block](https://github.com/nataliapc/makeTSX/wiki/Tutorial-How-to-generate-TSX-files#14-the-new-4b-block).
///
/// This block type facilitates encoding using the [Kansas City Standard (KCS)](https://en.wikipedia.org/wiki/Kansas_City_standard),
/// as used by [TSX](https://tsx.eslamejor.com) files for the [MSX](https://en.wikipedia.org/wiki/MSX) platform.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct KansasCityStandardDataBlock {
    #[br(temp)]
    #[bw(calc = payload.len() as u32 + 12)]
    length: u32,                  // Block length (without these four bytes)
    #[br(assert(length >= 12, "block length {} < 12", length))]
    pub pause: u16,               // Pause after this block (ms)
    pub length_pulse_pilot: u16,  // Duration of a PILOT pulse in T-states {same as ONE pulse}
    pub length_tone_pilot: u16,   // Number of pulses in the pilot tone
    pub length_pulse_zero: u16,   // Duration of a ZERO pulse in T-states
    pub length_pulse_one: u16,    // Duration of a ONE pulse in T-states
    #[br(map = |raw: u16| KCSBitByteConfig::from(raw))]
    #[bw(map = |bits: &KCSBitByteConfig| u16::from(*bits))]
    pub bit_byte_config: KCSBitByteConfig,
    #[br(args(8, length as usize - 12))]
    pub payload: DataPayload,
}

impl KansasCityStandardDataBlock {
    pub fn new() -> Self {
        Self {
            pause: 2000,
            length_pulse_pilot: 729,
            length_tone_pilot: 30720,
            length_pulse_zero: 1458,
            length_pulse_one: 729,
            bit_byte_config: KCSBitByteConfig::default(),
            payload: DataPayload::default(),
        }
    }
}

impl Default for KansasCityStandardDataBlock {
    fn default() -> Self { Self::new() }
}

impl fmt::Display for KansasCityStandardDataBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KansasCityStandardDataBlock: {:5} bytes, pause {:5}ms (pilot: {}*{}; 0/1: {}/{}; {})",
            self.payload.len(),
            self.pause,
            self.length_pulse_pilot,
            self.length_tone_pilot,
            self.length_pulse_zero,
            self.length_pulse_one,
            self.bit_byte_config,
        )
    }
}

impl Block for KansasCityStandardDataBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::KansasCityStandardDataBlock;
    }

    fn get_waveforms(&self, config: Arc<Config>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let pilot_source = PilotWaveform::new(
            config.clone(),
            self.length_pulse_pilot,
            self.length_tone_pilot,
            start_pulse_high,
        );
        let data_source = KansasCityStandardDataWaveform::new(
            config.clone(),
            self.length_pulse_zero,
            self.length_pulse_one,
            self.bit_byte_config.clone(),
            self.payload.clone(),
            if self.length_tone_pilot % 2 == 0 { start_pulse_high } else { !start_pulse_high },
        );
        let pause_source = PauseWaveform::new(config.clone(), self.pause, PauseType::Zero);

        return vec![Box::new(pilot_source), Box::new(data_source), Box::new(pause_source)];
    }

    fn next_block_start_pulse_high(&self, _config: Arc<Config>, self_start_pulse_high: bool) -> bool {
        if self.pause > 0 {
            return true;
        }

        return if self.length_tone_pilot % 2 == 0 { self_start_pulse_high } else { !self_start_pulse_high };
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn extended_display(&self, out: &mut dyn ExtendedDisplayCollector) {
        if let Some(payload) = self.payload.as_payload() {
            out.push(&format!("{}", payload));
        }
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
