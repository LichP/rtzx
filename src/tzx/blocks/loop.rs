use binrw::{
    binrw,
};
use std::any::Any;
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

/// A [Loop start](https://worldofspectrum.net/TZXformat.html#LOOPSTART) block.
/// Parsed, but currently unsupported.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct LoopStart {
    repititions: u16,
}

impl fmt::Display for LoopStart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LoopStart: {} repititions", self.repititions)
    }
}

impl Block for LoopStart {
    fn r#type(&self) -> BlockType {
        return BlockType::LoopStart;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// A [Loop end](https://worldofspectrum.net/TZXformat.html#LOOPEND) block.
/// Parsed, but currently unsupported.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct LoopEnd {
}

impl fmt::Display for LoopEnd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LoopEnd")
    }
}

impl Block for LoopEnd {
    fn r#type(&self) -> BlockType {
        return BlockType::LoopEnd;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
