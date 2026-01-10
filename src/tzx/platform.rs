//! TZX platforms.

use clap::ValueEnum;
use std::path::PathBuf;
use strum_macros::Display;

/// The standard t cycle length used by TZX files. All lengths measured in t cycles are specified in counts of
/// ZX Spectrum t cycles corresponding to the ZX Spectrum clock speed of 3.5MHz.
const T_CYCLE_LENGTH: f64 = 1.0 / 3500000.0;

/// Represents the platform targetted by the TZX data.
#[derive(Clone, Copy, Debug, Default, Display, Eq, PartialEq, Hash, ValueEnum)]
pub enum Platform {
    /// CDT files encode data for the Amstrad CPC.
    AmstradCPC,
    /// TZX files encode data for the ZX Spectrum.
    #[default]
    ZXSpectrum,
}

impl Platform {
    /// Determines the platform from the file extension of the given path.
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "cdt" => Some(Platform::AmstradCPC),
            "tzx" => Some(Platform::ZXSpectrum),
            _ => None,
        }
    }

    /// Determines the t cycle multiplier to use for creating TZX files from recordings.
    ///
    /// The Amstrad CPC has a clock speed of 4MHz, so all timings measured in CPC t cycles must be multiplied
    /// by multiplied by 4.0/3.5 for encoding to TZX using ZX Spectrum 3.5MHz t cycles.
    pub fn t_cycle_multiplier_record(&self) -> f64 {
        match self {
            Platform::AmstradCPC => 4.0 / 3.5,
            Platform::ZXSpectrum => 1.0,
        }
    }

    /// Returns the length of a t cycle in seconds as modified the given playback speed percentage.
    pub fn t_cycle_secs_playback(&self, playback_speed_percent: i32) -> f64 { T_CYCLE_LENGTH * (100 + playback_speed_percent) as f64 / 100.0 }
}
