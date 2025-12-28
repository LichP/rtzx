use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::{
    ExtendedDisplayCollector,
    blocks::{Block, BlockType}
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct SelectBlock {
    length: u16,
    #[br(if(length != 0, 0))]
    entry_count: u8,
    #[br(count = entry_count)]
    entries: Vec<SelectBlockEntry>
}

impl fmt::Display for SelectBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SelectBlock: {} selections", self.entry_count)
    }
}

impl Block for SelectBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::SelectBlock;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn extended_display(&self, out: &mut dyn ExtendedDisplayCollector) {
        for entry in &self.entries {
            out.push(entry);
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct SelectBlockEntry {
    offset: i16,
    length: u8,
    #[br(count = length)]
    text: Vec<u8>
}

impl fmt::Display for SelectBlockEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = String::from_utf8_lossy(&self.text);
        write!(f, "{} (offset: {})", description, self.offset)
    }
}
