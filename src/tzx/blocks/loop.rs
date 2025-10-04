use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

#[binrw]
#[brw(little)]
#[derive(Clone)]
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
}

#[binrw]
#[brw(little)]
#[derive(Clone)]
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
}
