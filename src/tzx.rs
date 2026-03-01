pub mod blocks;
pub mod config;
pub mod data;
pub mod header;
pub mod tap;
pub mod tzx_data;
pub mod platform;
pub mod player;
pub mod recovery_enum;
pub mod waveforms;

pub use config::Config;
pub use header::Header;
pub use tap::TapData;
pub use tzx_data::TzxData;
pub use platform::Platform;
pub use player::Player;
pub use recovery_enum::RecoveryEnum;

use binrw::BinResult;
use std::convert::From;
use std::fmt;
use std::io::{Read, Seek};

/// Facilitates gathering additional pieces of information for display in contexts
/// where more detail is desired.
pub trait ExtendedDisplayCollector {
    /// Push a piece of displayable data to the collector.
    ///
    /// Callers are free to push as many pieces of information as they like, for example
    /// for each entry in a collection.
    ///
    /// Trait implementations are responsible for displaying the information received from
    /// the sender in whatever way they see fit, e.g. by directly printing or collecting to a
    /// vec for later rendering in a UI.
    fn push(&mut self, piece: &dyn std::fmt::Display);
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum TapeDataFileType {
    Cdt,
    Tap,
    Tsx,
    #[default]
    Tzx,
}

impl fmt::Display for TapeDataFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_string = match self {
            TapeDataFileType::Cdt => "cdt",
            TapeDataFileType::Tap => "tap",
            TapeDataFileType::Tsx => "tsx",
            TapeDataFileType::Tzx => "tzx",
        };
        write!(f, "{}", type_string.to_uppercase())
    }
}

impl From<&str> for TapeDataFileType {
    fn from(extension: &str) -> Self {
        match extension {
            "cdt" => TapeDataFileType::Cdt,
            "tap" => TapeDataFileType::Tap,
            "tsx" => TapeDataFileType::Tsx,
            "tzx" => TapeDataFileType::Tzx,
            _ => TapeDataFileType::Tzx,
        }
    }
}

impl From<Option<&str>> for TapeDataFileType {
    fn from(extension: Option<&str>) -> Self {
        match extension {
            Some(ext_string) => Self::from(ext_string),
            _ => TapeDataFileType::Tzx,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TapeDataFile {
    pub file_type: TapeDataFileType,
    pub tzx_data: Option<TzxData>,
    pub tap_data: Option<TapData>,
}

impl TapeDataFile {
    pub fn read_as<R: Read + Seek>(reader: &mut R, file_type: TapeDataFileType) -> BinResult<Self> {
        match file_type {
            TapeDataFileType::Cdt | TapeDataFileType::Tsx | TapeDataFileType::Tzx => {
                let tzx_data = TzxData::read(reader)?;
                Ok(TapeDataFile { file_type, tzx_data: Some(tzx_data), tap_data: None })
            }
            TapeDataFileType::Tap => {
                let tap_data = TapData::read(reader)?;
                Ok(TapeDataFile { file_type, tzx_data: Some(tap_data.clone().into()), tap_data: Some(tap_data) })
            }
        }
    }
}
