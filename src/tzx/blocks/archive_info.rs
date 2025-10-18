use binrw::{
    binrw,
    BinRead, BinWrite,
};
use std::fmt;
use strum_macros::Display;
use crate::tzx::blocks::{Block, BlockType, BlockExtendedDisplayCollector};
use crate::tzx::RecoveryEnum;

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct ArchiveInfo {
    length: u16,
    #[br(if(length > 0, 0))]
    entry_count: u8,
    #[br(count = entry_count)]
    entries: Vec<ArchiveInfoEntry>
}

impl fmt::Display for ArchiveInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ArchiveInfo: {} entries", self.entry_count)
    }
}

impl Block for ArchiveInfo {
    fn r#type(&self) -> BlockType {
        return BlockType::ArchiveInfo;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn extended_display(&self, out: &mut dyn BlockExtendedDisplayCollector) {
        for entry in &self.entries {
            out.push(entry);
        }
    }
}

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum ArchiveInfoEntryType {
    FullTitle = 0x00,
    SoftwareHousePublisher = 0x01,
    Author = 0x02,
    YearPublished = 0x03,
    Language = 0x04,
    GameUtilityType = 0x05,
    Price = 0x06,
    ProtectionSchemeLoader = 0x07,
    Origin = 0x08,
    Comment = 0xff,
}

impl From<ArchiveInfoEntryType> for u8 {
    fn from(v: ArchiveInfoEntryType) -> u8 {
        v as u8
    }
}

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
pub struct ArchiveInfoEntry {
    entry_type: RecoveryEnum<ArchiveInfoEntryType, u8>,
    length: u8,
    #[br(count = length)]
    text: Vec<u8>
}

impl fmt::Display for ArchiveInfoEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = String::from_utf8_lossy(&self.text);
        write!(f, "{}: {}", match self.entry_type {
            RecoveryEnum::Known(entry_type) => format!("{}", entry_type),
            RecoveryEnum::Unknown(value) => format!("{}", value),
        }, description)
    }
}
