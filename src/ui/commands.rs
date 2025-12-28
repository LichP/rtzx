pub mod convert;
pub mod inspect;
pub mod play;

pub use convert::run_convert;
pub use inspect::run_inspect;
pub use play::run_play;

use clap::{Args, Subcommand};
use rodio::SampleRate;
use std::path::PathBuf;

use crate::tzx::{Config, Platform};

#[derive(Subcommand)]
pub enum Commands {
    /// Convert a tape file to wav
    Convert(ConvertArgs),
    /// Inspect a tape file
    Inspect(InspectArgs),
    /// Play a tape file
    Play(PlayArgs),
}

impl Commands {
    pub fn file_name(&self) -> Option<PathBuf> {
        match self {
            Commands::Inspect(args) => Some(args.file.file_name.clone()),
            Commands::Play(args) => Some(args.file.file_name.clone()),
            Commands::Convert(args) => Some(args.file.file_name.clone()),
        }
    }

    pub fn config(&self) -> Config {
        match self {
            Commands::Play(args) => args.config.to_config(args.file.file_name.clone()),
            Commands::Convert(args) => args.config.to_config(args.file.file_name.clone()),
            _ => Config::builder().build(),
        }
    }
}

#[derive(Args)]
pub struct ConfigArgs {
    /// The platform. Determined automatically from the filename if not supplied.
    ///
    /// Does not affect playback: use the --playback-duration-percent option to
    /// adjust playback speed.
    #[arg(short, long, value_enum)]
    platform: Option<Platform>,

    /// The sample rate to use for playback. Defaults to 44100 if not supplied.
    #[arg(short, long)]
    sample_rate: Option<SampleRate>,

    /// The length of the playback buffer in ms. Defaults to 200ms if not supplied.
    ///
    /// Only used by play mode - does not affect convert.
    #[arg(short, long)]
    buffer_length_ms: Option<u32>,

    /// Playback speed adjustment: a positive or negative integer percentage.
    ///
    /// Positive numbers increase playback duration. Defaults to zero (standard playback
    /// speed) if not supplied.
    #[arg(short = 'd', long)]
    playback_duration_percent: Option<i32>,
}

impl ConfigArgs {
    pub fn to_config(&self, file_name: PathBuf) -> Config {
        return Config::builder()
            .maybe_platform(self.platform.clone().or(Platform::from_path(file_name)))
            .maybe_sample_rate(self.sample_rate)
            .maybe_buffer_length_ms(self.buffer_length_ms)
            .maybe_playback_duration_percent(self.playback_duration_percent)
            .build();
    }
}

// Todo: ensure Inspect play still sniffs platform from filename?
#[derive(Args)]
pub struct FileArgs {
    /// The tape file (tzx / cdt)
    file_name: PathBuf,
}

#[derive(Args)]
pub struct ConvertArgs {
    #[command(flatten)]
    config: ConfigArgs,

    #[command(flatten)]
    file: FileArgs,

    /// The filename to output to. Defaults to the same name as the tzx / cdt file with a .wav extension if not supplied.
    #[arg(short, long)]
    output_file_name: Option<PathBuf>,
}

#[derive(Args)]
pub struct InspectArgs {
    #[command(flatten)]
    file: FileArgs,

    /// Include waveforms in the inspection.
    #[arg(short, long, default_value_t = false)]
    pub waveforms: bool
}

#[derive(Args)]
pub struct PlayArgs {
    #[command(flatten)]
    config: ConfigArgs,

    #[command(flatten)]
    file: FileArgs,
}
