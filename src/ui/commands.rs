pub mod convert;
pub mod inspect;
pub mod play;

pub use convert::run_convert;
pub use inspect::run_inspect;
pub use play::run_play;

use clap::{Args, Subcommand};

use crate::tzx::Machine;

#[derive(Subcommand)]
pub enum Commands {
    /// Convert a tape file to wav
    Convert(ConvertArgs),
    /// Inspect a tape file
    Inspect(FileArgs),
    /// Play a tape file
    Play(FileArgs),
}

impl Commands {
    pub fn file_name(&self) -> Option<&str> {
        match self {
            Commands::Inspect(args) |
            Commands::Play(args)
              => Some(&args.file_name),
            Commands::Convert(args) => Some(&args.file.file_name),
        }
    }

    pub fn machine(&self) -> Option<Machine> {
        match self {
            Commands::Inspect(args) | Commands::Play(args) => args.machine.clone(),
            Commands::Convert(args) => args.file.machine.clone(),
        }
    }
}

#[derive(Args)]
pub struct FileArgs {
    /// The machine to use for playback timings. Determined automatically from the filename if not supplied.
    #[arg(short, long, value_enum)]
    machine: Option<Machine>,
    /// The tape file (tzx / cdt)
    file_name: String,
}

#[derive(Args)]
pub struct ConvertArgs {
    #[command(flatten)]
    file: FileArgs,

    #[arg(short, long)]
    output_file_name: String,
}
