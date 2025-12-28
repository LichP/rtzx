use std::io;
use std::path::Path;

use crate::tzx::{
    Config,
    ExtendedDisplayCollector,
    TzxData,
};

struct InspectPrintCollector;

impl ExtendedDisplayCollector for InspectPrintCollector {
    fn push(&mut self, item: &dyn std::fmt::Display) {
        println!("               {}", item);
    }
}

pub fn run_inspect(path: &Path, config: &Config, waveforms: bool, tzx_data: &TzxData) -> io::Result<()> {
    println!("TZX file: {}", path.display());
    println!("Platform:  {:?}", config.platform);
    println!("Header:   {}", tzx_data.header);

    let mut printer = InspectPrintCollector;

    let config = std::sync::Arc::new(config.clone());

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

    return Ok(());
}
