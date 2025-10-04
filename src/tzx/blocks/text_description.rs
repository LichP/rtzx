use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct TextDescription {
    length: u8,
    #[br(count = length)]
    text: Vec<u8>
}

impl fmt::Display for TextDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = String::from_utf8_lossy(&self.text);
        write!(f, "TextDescription: {}", description)
    }
}

impl Block for TextDescription {
    fn r#type(&self) -> BlockType {
        return BlockType::TextDescription;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct MessageBlock {
    display_for_secs: u8,
    length: u8,
    #[br(count = length)]
    text: Vec<u8>
}

impl fmt::Display for MessageBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = String::from_utf8_lossy(&self.text);
        write!(f, "MessageBlock: {}", description)
    }
}

impl Block for MessageBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::TextDescription;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
