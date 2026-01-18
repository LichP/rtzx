use binrw::{
    binrw,
};
use bitvec::prelude::*;
use strum_macros::Display;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::tzx::{
    Config,
    ExtendedDisplayCollector,
    blocks::{Block, BlockType},
    data::DataPayload,
    waveforms::{
        GeneralizedWaveform,
        PauseType,
        PauseWaveform,
        Waveform,
    },
};

/// Represents the desired polarity state of the first pulse in a symbol.
#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Default, Display, Debug, Eq, PartialEq, Hash)]
pub enum SymbolPolarity {
    /// The first pulse should be the opposite polarity of the preceding pulse.
    #[default]
    Opposite = 0,
    /// The first pulse should be the same polarity as the preceding pulse.
    Same = 1,
    /// The first pulse should always be low.
    ForceLow = 2,
    /// The first pulse should always be high.
    ForceHigh = 3,
}

impl SymbolPolarity {
    /// Returns the polarity to use for the next pulse when starting a new symbol given the polarity of the current pulse.
    pub fn next_polarity(&self, current_polarity: bool) -> bool {
        match self {
            SymbolPolarity::Opposite => !current_polarity,
            SymbolPolarity::Same => current_polarity,
            SymbolPolarity::ForceLow => false,
            SymbolPolarity::ForceHigh => true,
        }
    }
}

/// A symbol definition.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
#[br(import(max_pulses: u8))]
pub struct SymbolDefinition {
    /// The polarity for the first pulse in the symbol.
    /// This corresponds to the flags field in the spec, potentially other bits could be used for other flags in future?
    pub polarity: SymbolPolarity,
    /// The lengths of each pulse in the symbol. A length of zero indicates that the symbol has ended.
    #[br(count = max_pulses)]
    pub pulses: Vec<u16>,
}

impl SymbolDefinition {
    pub fn new() -> Self { SymbolDefinition::default() }
}

impl fmt::Display for SymbolDefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {:?}]", self.polarity, self.pulses)
    }
}

/// A wrapper struct to facilitate displaying a `Vec<SymbolDefinition>`.
pub struct SymbolDefinitionVecDisplay(Vec<SymbolDefinition>);

impl fmt::Display for SymbolDefinitionVecDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for i in 0..std::cmp::min(self.0.len(), 4) {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", i, self.0[i])?;
        }
        if self.0.len() >= 4 {
            write!(f, ", ...")?;
        }
        write!(f, "]")
    }
}

// An entry in the pilot sequence run-length encoding.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, Default, Hash)]
pub struct PilotRLE {
    /// A key identifying an entry in the pilot symbol table.
    symbol: u8,
    /// The number of times the symbol should be repeated.
    repetitions: u16,
}

impl PilotRLE {
    pub fn new(symbol: u8, repetitions: u16) -> Self { PilotRLE { symbol, repetitions } }
}

impl fmt::Display for PilotRLE {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} x {}", self.symbol, self.repetitions)
    }
}

/// A wrapper struct to facilitate displaying an `Arc<Vec<PilotRLE>>`.
pub struct PilotRLEVecDisplay(Arc<Vec<PilotRLE>>);

impl fmt::Display for PilotRLEVecDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for i in 0..std::cmp::min(self.0.len(), 10) {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", i, self.0[i])?;
        }
        if self.0.len() >= 10 {
            write!(f, ", ...")?;
        }
        write!(f, "]")
    }
}

/// A [Generalized Data Block](https://worldofspectrum.net/TZXformat.html#GENDATA).
///
/// This is a somewhat complex block encoding consisting of symbol definition tables for pilot and or data streams,
/// an optional run-length encoding of the pilot, and an optional data stream comprising a series of chunks of
/// 1 to 8 bits with each chunk representing a key to look up a data stream symbol.
///
/// Generalized data block support is currently considered experimental.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
#[br(import())]
pub struct GeneralizedDataBlock {
    length: u32,    // Block length (without these four bytes)
    #[br(assert(length >= 14, "block length {} < 14", length))]
    pause: u16,     // Pause after this block (ms)
    totp: u32,      // Total number of symbols in pilot/sync block (can be 0)
    npp: u8,        // Maximum number of pulses per pilot/sync symbol
    asp: u8,        // Number of pilot/sync symbols in the alphabet table (0=256)
    totd: u32,      // Total number of symbols in data stream (can be 0)
    npd: u8,        // Maximum number of pulses per data symbol
    asd: u8,        // Number of data symbols in the alphabet table (0=256)
    #[br(args {
        count: if totp > 0 { if asp == 0 { 256 } else { asp as usize }} else { 0 },
        inner: (npp, ),
    })]
    symbols_pilot: Vec<SymbolDefinition>,   // Pilot and sync symbols definition table
    #[br(count = totp, map = |v: Vec<PilotRLE>| Arc::new(v))]
    #[bw(map = |arc: &Arc<Vec<PilotRLE>>| &**arc)]
    pilot_data: Arc<Vec<PilotRLE>>,         // Pilot and sync data stream
    #[br(args {
        count: if totd > 0 { if asd == 0 { 256 } else { asd as usize }} else { 0 },
        inner: (npd, )
    })]
    symbols_data: Vec<SymbolDefinition>,    // Data symbols definition table
    #[br(
        count = length as usize
            - 14
            - (if totp > 0 { (if asp == 0 { 256 } else { asp as usize } * (npp as usize * 2 + 1)) } else { 0 })
            - (totp as usize * 3)
            - (if totd > 0 { (if asd == 0 { 256 } else { asd as usize } * (npd as usize * 2 + 1)) } else { 0 }),
        map = |v: Vec<u8>| Arc::new(v)
    )]
    #[bw(map = |arc: &Arc<Vec<u8>>| &**arc)]
    data: Arc<Vec<u8>>,                     // Data stream
}

impl fmt::Display for GeneralizedDataBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GeneralizedDataBlock: {:5} bytes, pause {:5}ms (pilot tot/np/as: {}/{}/{}; data tot/np/as: {}/{}/{})",
            self.data.len(),
            self.pause,
            self.totp,
            self.npp,
            self.asp,
            self.totd,
            self.npd,
            self.asd,
        )
    }
}

impl GeneralizedDataBlock {
    pub fn pilot_data_payload(&self) -> DataPayload {
        let mut data = bitvec![u8, Msb0;];
        for entry in self.pilot_data.as_slice() {
            let symbol_bits = entry.symbol.view_bits::<Msb0>();
            for _ in 0..entry.repetitions {
                data.extend(&symbol_bits[(8 - self.symbols_pilot.len().ilog2() as usize)..8]);
            }
        }
        DataPayload::new((8 - data.len() % self.symbols_pilot.len().ilog2() as usize) as u8, data.len() as u32, Arc::new(data.into_vec()))
    }
}

impl Block for GeneralizedDataBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::GeneralizedDataBlock;
    }

    fn get_waveforms(&self, config: Arc<Config>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let mut pilot_source: Option<GeneralizedWaveform> = None;
        if self.totp > 0 {
            let pilot_payload = self.pilot_data_payload();
            pilot_source = Some(GeneralizedWaveform::new(
                config.clone(),
                Arc::new(self.symbols_pilot.clone()),
                pilot_payload,
                start_pulse_high,
            ));
        }

        let data_used_bits = 8 - (self.data.len() * 8).saturating_sub(self.symbols_data.len().ilog2() as usize * self.totd as usize) as u8;
        let data_payload = DataPayload::new(data_used_bits, self.data.len() as u32, self.data.clone());
        let data_source = GeneralizedWaveform::new(
            config.clone(),
            Arc::new(self.symbols_data.clone()),
            data_payload,
            if self.totp > 0 { pilot_source.clone().unwrap().last_pulse_high() } else { start_pulse_high },
        );
        let pause_source = PauseWaveform::new(config.clone(), self.pause, PauseType::Zero);

        if self.totp > 0 {
            return vec![Box::new(pilot_source.unwrap()), Box::new(data_source), Box::new(pause_source)];
        } else {
            return vec![Box::new(data_source), Box::new(pause_source)];
        }
    }

    fn next_block_start_pulse_high(&self, _config: Arc<Config>, _self_start_pulse_high: bool) -> bool { true }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn extended_display(&self, out: &mut dyn ExtendedDisplayCollector) {
        if self.symbols_pilot.len() > 0 {
            out.push(&format!("Pilot Symbols: {}", SymbolDefinitionVecDisplay(self.symbols_pilot.clone())))
        }
        if self.pilot_data.len() > 0 {
            out.push(&format!("Pilot Data: {}", PilotRLEVecDisplay(self.pilot_data.clone())))
        }
        if self.symbols_data.len() > 0 {
            out.push(&format!("Data Symbols: {}", SymbolDefinitionVecDisplay(self.symbols_data.clone())))
        }
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
