use std::io;
use std::path::Path;

use crate::tzx::{
    Config,
    TzxData,
    blocks::BlockExtendedDisplayCollector,
};

struct InspectPrintCollector;

impl BlockExtendedDisplayCollector for InspectPrintCollector {
    fn push(&mut self, item: &dyn std::fmt::Display) {
        println!("               {}", item);
    }
}

pub fn run_inspect(path: &Path, config: &Config, tzx_data: &TzxData) -> io::Result<()> {
    println!("TZX file: {}", path.display());
    println!("Platform:  {:?}", config.platform);
    println!("Header:   {}", tzx_data.header);

    let mut printer = InspectPrintCollector;

    for (index, block) in tzx_data.blocks.iter().enumerate() {
        println!("Block {:3}/{:3}: {}", index + 1, tzx_data.blocks.len(), block);
        block.extended_display(&mut printer);
    }
    return Ok(());
}
