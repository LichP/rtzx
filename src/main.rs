use clap::Parser;
use std::fs::File;
use std::io;

use crate::tzx::TzxData;
use crate::ui::commands::{
    Commands,
    convert::run_convert,
    inspect::run_inspect,
    play::run_play,
};

pub mod tzx;
pub mod ui;

/// rtzx: A utility for interacting with TZX / CDT tape files.
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // Create a path to the desired file
    let file_name = &cli.command.as_ref().and_then(|cmd| cmd.file_name()).expect("Filename not supplied");
    let display = file_name.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let file = match File::open(file_name) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let config = &cli.command.as_ref().and_then(|cmd| Some(cmd.config())).unwrap();

    let tzx_data = TzxData::parse_from(file);

    return match &cli.command {
        Some(Commands::Inspect(args)) => run_inspect(file_name, &config, args.waveforms, &tzx_data),
        Some(Commands::Convert(args)) => run_convert(&args, &config, &tzx_data),
        Some(Commands::Play(_)) => run_play(file_name, &config, &tzx_data),
        None => Ok(()),
    };
}
