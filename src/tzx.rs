pub mod blocks;
pub mod header;
pub mod data;
pub mod playlist;
pub mod waveforms;

pub use header::Header;
pub use data::TzxData;
pub use playlist::Playlist;

use clap::ValueEnum;
use std::path::Path;

const T_CYCLE_LENGTH: f64 = 1.0 / 3500000.0;

#[derive(Clone, Debug, ValueEnum)]
pub enum Machine {
    AmstradCPC,
    ZXSpectrum,
}

impl Machine {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "cdt" => Some(Machine::AmstradCPC),
            "tzx" => Some(Machine::ZXSpectrum),
            _ => None,
        }
    }

    pub fn t_cycle_multiplier(&self) -> f64 {
        match self {
            Machine::AmstradCPC => 4.0 / 3.5,
            Machine::ZXSpectrum => 1.0,
        }
    }

    pub fn t_cycle_secs(&self) -> f64 { T_CYCLE_LENGTH * self.t_cycle_multiplier() }
}