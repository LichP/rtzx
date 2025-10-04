use clap::Parser;
use std::fs::File;
use std::io;
use std::path::Path;

use crate::tzx::{
    Machine, TzxData,
};
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
    let file_name = &cli.command.as_ref().and_then(|cmd| cmd.file_name());
    let path = Path::new(file_name.expect("File name not supplied"));
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let machine = match &cli.command.as_ref().and_then(|cmd| cmd.machine()) {
        Some(machine) => machine.clone(),
        None => Machine::from_path(path).expect("Unable to determine machine type from path"),
    };
    let tzx_data = TzxData::parse_from(file);

    return match &cli.command {
        Some(Commands::Inspect(_)) => run_inspect(path, &machine, &tzx_data),
        Some(Commands::Convert(args)) => run_convert(&args, &machine, &tzx_data),
        Some(Commands::Play(_)) => run_play(path, &machine, &tzx_data),
        None => Ok(()),
    };
}
