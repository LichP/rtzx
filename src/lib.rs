//! A library for parsing [ZX Spectrum](https://en.wikipedia.org/wiki/ZX_Spectrum) `.tzx` and
//! [Amstrad CPC](https://en.wikipedia.org/wiki/Amstrad_CPC) `.cdt` tape data files, and converting
//! encoded tape data to waveform sample data for conversion to WAV or direct playback.
//!
//! To parse a TZX/CDT data, use [`rtzx::TzxData::parse_from()`](crate::tzx::TzxData::parse_from).
//!
//! Example:
//!
//! ```
//! let file = match File::open("some-zx-spectrum-tape.tzx") {
//!     Err(why) => panic!("Couldn't open file: {}", why),
//!     Ok(file) => file,
//! };
//! let tzx_data = TzxData::parse_from(file);
//! ```
//!
//! The [`rtzx::TzxData`](crate::tzx::TzxData) struct contains two fields,
//! the [`.header`](crate::tzx::TzxData::header) for specifying the TZX version, and
//! [`.blocks`](crate::tzx::TzxData::blocks), which a vector containing all blocks parsed from the
//! TZX data.
//!
//! Parsing a TZX/CDT file does not require configuration, however conversion / playback will need
//! [`rtzx::Config`](crate::tzx::Config) to set the sample rate, playback speed, etc.
//!
//! To obtain sample data for conversion / playback, the idea is to loop over the blocks and use
//! [`.get_waveforms()`](crate::tzx::blocks::Block::get_waveforms). Waveform structs implement
//! the [`Waveform`](crate::tzx::waveforms::Waveform) trait and can either be iterated dirctly for
//! conversion to WAV
//! (e.g. [`rtzx::ui::commands::convert::run_convert`](crate::ui::commands::run_convert))
//! or used as [Source](https://docs.rs/rodio/latest/rodio/source/trait.Source.html)s for playback
//! with [Rodio](https://docs.rs/rodio/latest/rodio/)

pub mod tzx;
pub mod ui;

pub use crate::tzx::{
    Config,
    Platform,
    TapeDataFile,
    TapeDataFileType,
    TzxData,
};
