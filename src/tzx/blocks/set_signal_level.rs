use binrw::{
    binrw,
};
use std::fmt;
use std::sync::Arc;

use crate::tzx::{
    Config,
    blocks::{Block, BlockType}
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct SetSignalLevel {
    pub length: u32,
    #[br(if(length != 0, 1))]
    signal_level: u8,
    #[br(count = if length > 0 { length - 1 } else { 0 })]
    payload: Vec<u8>,
}

impl fmt::Display for SetSignalLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SetSignalLevel: {}", if self.signal_level == 0 { "low" } else { "high"})
    }
}

impl Block for SetSignalLevel {
    fn r#type(&self) -> BlockType {
        return BlockType::SetSignalLevel;
    }

    fn next_block_start_pulse_high(&self, _config: Arc<Config>, _self_start_pulse_high: bool) -> bool { self.signal_level != 0 }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}