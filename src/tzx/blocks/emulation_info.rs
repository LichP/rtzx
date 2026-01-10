use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

/// A deprecated [Emulation info](https://worldofspectrum.net/TZXformat.html#EMUINFO) block.
/// Parsed, but unsupported other than for presentation of encoded data.
#[binrw]
#[brw(little)]
#[derive(Clone, Copy, Debug)]
pub struct EmulationInfo {
    flags: u16,
    refresh_delay: u8,
    interupt_frequency: u16,
    reserved_one: u8,
    reserved_two: u8,
    reserved_three: u8
}

impl fmt::Display for EmulationInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EmulationInfo: {:#b} {:02X} {:02X} {:02X} {:02X} {:02X}",
            self.flags,
            self.refresh_delay,
            self.interupt_frequency,
            self.reserved_one,
            self.reserved_two,
            self.reserved_three,
        )
    }
}

impl Block for EmulationInfo {
    fn r#type(&self) -> BlockType {
        return BlockType::EmulationInfo;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
