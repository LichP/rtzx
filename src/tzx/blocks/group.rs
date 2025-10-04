use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

#[binrw]
#[brw(little)]
#[derive(Clone)]
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
}

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
}
