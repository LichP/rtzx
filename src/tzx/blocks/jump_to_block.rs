use binrw::{
    binrw,
};
use std::any::Any;
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

/// A [Jump to block](https://worldofspectrum.net/TZXformat.html#JUMPBLOCK) block.
/// Parsed, but currently unsupported.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
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
        return BlockType::JumpToBlock;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
