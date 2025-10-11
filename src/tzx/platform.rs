use clap::ValueEnum;
use std::path::PathBuf;

const T_CYCLE_LENGTH: f64 = 1.0 / 3500000.0;

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum Platform {
    AmstradCPC,
    #[default]
    ZXSpectrum,
}

impl Platform {
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

    pub fn t_cycle_multiplier(&self) -> f64 {
        match self {
            Platform::AmstradCPC => 4.0 / 3.5,
            Platform::ZXSpectrum => 1.0,
        }
    }

    pub fn t_cycle_secs(&self) -> f64 { T_CYCLE_LENGTH * self.t_cycle_multiplier() }
}
