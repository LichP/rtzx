use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct CustomInfoBlock {
    #[br(count = 16)]
    id: Vec<u8>,
    length: u32,
    #[br(count = length)]
    data: Vec<u8>,
}

impl fmt::Display for CustomInfoBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id_string = String::from_utf8_lossy(&self.id);
        write!(f, "CustomInfoBlock: {} :", id_string)?;
        for byte in &self.data {
            write!(f, " {:02X}", byte)?;
        }
        Ok(())
    }
}

impl Block for CustomInfoBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::CustomInfoBlock;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct InstructionsBlock {
    block_length: u32,
    #[br(count = if block_length == 0x7274736e { 11 } else { 0 })]
    padding: Vec<u8>,
    #[br(if(block_length == 0x7274736e, 0))]
    length: u32,
    #[br(count = if block_length == 0x7274736e { length } else { block_length } )]
    payload: Vec<u8>
}

impl fmt::Display for InstructionsBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let length = if self.block_length == 0x7274736e { self.length } else { self.block_length };
        let description = if self.block_length == 0x7274736e { String::from_utf8_lossy(&self.payload).to_string() } else { String::new() };
        write!(f, "InstructionsBlock: {} bytes (deprecated): {}", length, description)
    }
}

impl Block for InstructionsBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::InstructionsBlock;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
