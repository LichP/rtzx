use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct JumpToBlock {
    offset: i16,
}

impl fmt::Display for JumpToBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JumpToBlock: {}", self.offset)
    }
}

impl Block for JumpToBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::GroupStart;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
