use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct CallSequence {
    length: u16,
    #[br(count = length)]
    block_offsets: Vec<i16>,
}

impl fmt::Display for CallSequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CallSequence: {} blocks", self.length)
    }
}

impl Block for CallSequence {
    fn r#type(&self) -> BlockType {
        return BlockType::CallSequence;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct ReturnFromSequence {
}

impl fmt::Display for ReturnFromSequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ReturnFromSequence")
    }
}

impl Block for ReturnFromSequence {
    fn r#type(&self) -> BlockType {
        return BlockType::ReturnFromSequence;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
