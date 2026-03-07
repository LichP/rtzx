use binrw::{
    binrw,
    BinWrite,
};
use num_enum::TryFromPrimitive;
use strum_macros::Display;
use std::any::Any;
use std::borrow::Cow;
use std::fmt;
use std::io::Cursor;
use std::sync::Arc;

use crate::tzx::{
    blocks::{Block, KansasCityStandardDataBlock},
    data::DataPayload,
    tap::Payload,
};

/// A standard MSX tape block header.
///
/// MSX header blocks are 16 bytes long with no checksum. The first ten bytes are repeated and
/// indicate the file type. [read_payload] will read these ten bytes to validate payload is
/// indeed an MSX header, and will pass in the corresponding [MSXFileType] as an argument.
#[binrw]
#[brw(little)]
#[brw(import(file_type: MSXFileType))]
#[derive(Debug, Clone, Hash)]
pub struct MSXHeader {
    #[br(calc = file_type)]
    file_type: MSXFileType,
    #[br(ignore)]
    #[bw(calc = vec![*file_type as u8; 9])]
    file_type_repeat: Vec<u8>,
    #[br(count = 6)]
    filename: Vec<u8>,
}

impl MSXHeader {
    pub fn new(
        file_type: MSXFileType,
        filename: &str,
    ) -> Self {
        MSXHeader {
            file_type,
            filename: filename.as_bytes().to_vec(),
        }
    }

    pub fn encoded(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        self.write(&mut writer).unwrap();
        return writer.into_inner();
    }

    pub fn into_kansas_city_standard_data_block(&self) -> KansasCityStandardDataBlock {
        let mut kcsdb = KansasCityStandardDataBlock::new();
        kcsdb.pause = 1000;
        kcsdb.payload = self.into();
        kcsdb
    }

    fn filename(&self) -> Cow<'_, str> {
        let end = self.filename.iter().position(|&b| b == 0).unwrap_or(self.filename.len());
        String::from_utf8_lossy(&self.filename[..end])
    }
}

impl Default for MSXHeader {
    fn default() -> Self {
        MSXHeader::new(MSXFileType::default(), "")
    }
}

impl From<&MSXHeader> for DataPayload {
    fn from(value: &MSXHeader) -> Self
    {
        DataPayload::new(8, Arc::new(value.encoded()))
    }
}

impl Payload for MSXHeader {
    fn bytes(&self) -> Vec<u8> {
        return self.encoded();
    }

    fn clone_box(&self) -> Box<dyn Payload> {
        Box::new(self.clone())
    }

    fn into_block_box(self: Box<Self>) -> Box<dyn Block> {
        Box::new((*self).into_kansas_city_standard_data_block())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl fmt::Display for MSXHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MSXHeader: {:16} (type: {})",
            self.filename(),
            self.file_type,
        )
    }
}

/// MSX file type byte.
#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug, Default, Eq, PartialEq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum MSXFileType {
    Binary = 0xd0,
    Basic = 0xd3,
    #[default]
    Ascii = 0xea,
}
