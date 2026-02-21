use binrw::{
    binrw,
};
use std::any::Any;
use std::fmt;
use crate::tzx::{
    ExtendedDisplayCollector,
    blocks::{Block, BlockType}
};

/// A [Custom info](https://worldofspectrum.net/TZXformat.html#CUSTOMBLOCK) block.
/// Parsed, but unsupported other than for presentation of encoded bytes.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
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
        let data_string = String::from_utf8_lossy(&self.data);
        write!(f, "CustomInfoBlock: {} : {}", id_string, data_string)
    }
}

impl Block for CustomInfoBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::CustomInfoBlock;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn extended_display(&self, out: &mut dyn ExtendedDisplayCollector) {
        let mut chunk_count = 0;
        for chunk in self.data.chunks(16) {
            let mut chunk_string = format!("  {:04x}: ", chunk_count * 16);

            for (i, byte) in chunk.iter().enumerate() {
                if i % 16 == 8 { chunk_string.push(' ') }
                chunk_string.push_str(&format!("{:02x} ", byte));
            }
            if chunk.len() < 16 {
                if chunk.len() < 8 { chunk_string.push(' ') }
                chunk_string.push_str(&" ".repeat((16 - chunk.len()) * 3));
            }

            chunk_string.push_str("  |");
            for byte in chunk {
                chunk_string.push(to_ascii_or_dot(*byte));
            }
            if chunk.len() < 16 {
                chunk_string.push_str(&" ".repeat(16 - chunk.len()));
            }

            chunk_string.push('|');

            out.push(&chunk_string);
            chunk_count += 1;
        }
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// A badly encoded deprecated instructions block (see [custom info deprecated types](https://worldofspectrum.net/TZXformat.html#CUSTINFODPR)).
/// This has mainly been implemented for compatability with Kevin Thacker's test CDT which includes it, and on the off-chance
/// any very old TZX / CDT files actually use it.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct InstructionsBlock {
    block_length: u32,
    #[br(count = if block_length == 0x7274736e { 11 } else { 0 })]
    padding: Vec<u8>,
    #[br(if(block_length == 0x7274736e, 0))]
    #[bw(if(*block_length == 0x7274736e))]
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

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

fn to_ascii_or_dot(byte: u8) -> char {
    let c = byte as char;
    if c.is_ascii_graphic() || c == ' ' {
        c
    } else {
        '.'
    }
}
