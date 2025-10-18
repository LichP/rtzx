use binrw::{
    binrw,
};
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct SnapshotBlock {
    format: u8,
    #[br(parse_with = binrw::helpers::read_u24)]
    #[bw(write_with = binrw::helpers::write_u24)]
    length: u32,
    #[br(count = length)]
    data: Vec<u8>,
}

impl fmt::Display for SnapshotBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SnapshotBlock: {}, {} bytes",
            match self.format {
                0 => format!("{}", "Z80"),
                1 => format!("{}", "SNA"),
                other => format!("{}", other),
            },
            self.length
        )
    }
}

impl Block for SnapshotBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::SnapshotBlock;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
