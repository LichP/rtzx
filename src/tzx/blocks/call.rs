use binrw::{
    binrw,
};
use std::any::Any;
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

/// A [Call sequence](https://worldofspectrum.net/TZXformat.html#CALLSEQ) block.
/// Parsed, but unsupported.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
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

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// A [Return from sequence](https://worldofspectrum.net/TZXformat.html#RETURNSEQ) block.
/// Parsed, but not supported.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
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

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
