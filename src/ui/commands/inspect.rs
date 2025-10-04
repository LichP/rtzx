use std::io;
use std::path::Path;

use crate::tzx::{
    Machine,
    TzxData,
};

pub fn run_inspect(path: &Path, machine: &Machine, tzx_data: &TzxData) -> io::Result<()> {
    println!("TZX file: {}", path.display());
    println!("Machine:  {:?}", machine);
    println!("Header:   {}", tzx_data.header);

    for (index, block) in tzx_data.blocks.iter().enumerate() {
        println!("Block {:3}/{:3}: {}", index + 1, tzx_data.blocks.len(), block);
    }
    return Ok(());
}
