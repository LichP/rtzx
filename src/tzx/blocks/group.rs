use binrw::{
    binrw,
};
use std::any::Any;
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

/// A [Group start](https://worldofspectrum.net/TZXformat.html#GRPSTART) block.
/// Passively supported, in that it doesn't really do anything.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct GroupStart {
    length: u8,
    #[br(count = length)]
    text: Vec<u8>
}

impl fmt::Display for GroupStart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = String::from_utf8_lossy(&self.text);
        write!(f, "GroupStart: {}", description)
    }
}

impl Block for GroupStart {
    fn r#type(&self) -> BlockType {
        return BlockType::GroupStart;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// A [Group end](https://worldofspectrum.net/TZXformat.html#GRPEND) block.
/// Passively supported, in that it doesn't really do anything.
#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct GroupEnd {
}

impl fmt::Display for GroupEnd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GroupEnd")
    }
}

impl Block for GroupEnd {
    fn r#type(&self) -> BlockType {
        return BlockType::GroupEnd;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
