use clap::Parser;
use std::fs::File;
use std::io;

use rtzx::{TapeDataFile, TapeDataFileType};
use rtzx::ui::commands::{
    Commands,
    convert::run_convert,
    inspect::run_inspect,
    play::run_play,
};

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

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(file_name) {
        Err(why) => panic!("Couldn't open {:?}: {}", file_name, why),
        Ok(file) => file,
    };

    let config = &cli.command.as_ref().and_then(|cmd| Some(cmd.config())).unwrap();

    let file_type = TapeDataFileType::from(file_name.extension().and_then(|s| s.to_str()));

    let file_data = match TapeDataFile::read_as(&mut file, file_type) {
        Err(why) => panic!("Failed to parse {:?} as {}: {}", file_name, file_type, why),
        Ok(data) => data,
    };

    return match &cli.command {
        Some(Commands::Inspect(args)) => run_inspect(file_name, &config, args.waveforms, &file_data),
        Some(Commands::Convert(args)) => run_convert(&args, &config, &file_data),
        Some(Commands::Play(_)) => run_play(file_name, &config, &file_data),
        None => Ok(()),
    };
}
