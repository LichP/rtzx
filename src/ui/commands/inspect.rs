use std::io;
use std::path::Path;

use crate::{TapeDataFile, TapeDataFileType, tzx::{
    Config,
    ExtendedDisplayCollector,
}};

struct InspectPrintCollector;

impl ExtendedDisplayCollector for InspectPrintCollector {
    fn push(&mut self, item: &dyn std::fmt::Display) {
        println!("               {}", item);
    }
}

pub fn run_inspect(path: &Path, config: &Config, waveforms: bool, tape_data: &TapeDataFile) -> io::Result<()> {
    let mut printer = InspectPrintCollector;
    let config = std::sync::Arc::new(config.clone());

    println!("{} file: {}", tape_data.file_type, path.display());
    println!("Platform: {:?}", config.platform);

    match tape_data.file_type {
        TapeDataFileType::Cdt | TapeDataFileType::Tsx | TapeDataFileType::Tzx => {
            let tzx_data = tape_data.tzx_data.as_ref().expect("TZX data missing!");
            println!("Header:   {}", tzx_data.header);
            for (index, block) in tzx_data.blocks.iter().enumerate() {
                println!("Block {:3}/{:3}: {}", index + 1, tzx_data.blocks.len(), block);
                block.extended_display(&mut printer);

                if waveforms {
                    let waveforms = block.get_waveforms(config.clone(), true);
                    for waveform in waveforms {
                        println!("  Waveform: {}", waveform);
                    }
                }
            }
        },
        TapeDataFileType::Tap => {
            let tap_data = tape_data.tap_data.as_ref().expect("TAP data missing!");
            for (index, block) in tap_data.blocks.iter().enumerate() {
                println!("Block {:3}/{:3}: {}", index + 1, tap_data.blocks.len(), block);
            }
        }
    }

    return Ok(());
}
