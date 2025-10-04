use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct SetSignalLevel {
    pub length: u32,
    signal_level: u8,
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

    fn next_block_start_pulse_high(&self, _self_start_pulse_high: bool) -> bool { self.signal_level != 0 }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}