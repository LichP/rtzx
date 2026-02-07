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
    blocks::{Block, StandardSpeedDataBlock},
    data::DataPayload,
    tap::{Payload, XorReader, XorWriter}
};

/// A standard ZX Spectrum tape block header.
///
/// Spectrum header blocks are 17 bytes long, followed by one byte XOR checksum of all bytes in the
/// header (and technically the 0x00 flag byte, which does not impact on the XOR).
#[binrw]
#[brw(little)]
#[br(stream = r, map_stream = |s| XorReader::new(s, SpectrumFlag::SpectrumHeader as u8))]
#[bw(stream = w, map_stream = |s| XorWriter::new(s, SpectrumFlag::SpectrumHeader as u8))]
#[derive(Debug, Clone, Hash)]
pub struct SpectrumHeader {
    file_type: SpectrumFileType,
    #[br(count = 10)]
    filename: Vec<u8>,
    data_length: u16,
    parameter1: u16,
    parameter2: u16,
    #[br(temp, assert(checksum == r.xor() ^ checksum, "XOR mismatch: expected {:02X}, got {:02X}", r.xor() ^ checksum, checksum))]
    #[bw(calc(w.xor()))]
    checksum: u8,
}

impl SpectrumHeader {
    pub fn new(
        file_type: SpectrumFileType,
        filename: &str,
        data_length: u16,
        parameter1: u16,
        parameter2: u16,
    ) -> Self {
        SpectrumHeader {
            file_type,
            filename: filename.as_bytes().to_vec(),
            data_length,
            parameter1,
            parameter2,
        }
    }

    pub fn encoded(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        SpectrumFlag::SpectrumHeader.write(&mut writer).unwrap();
        self.write(&mut writer).unwrap();
        return writer.into_inner();
    }

    pub fn into_standard_speed_data_block(&self) -> StandardSpeedDataBlock {
        let mut ssdb = StandardSpeedDataBlock::new();
        ssdb.pause = 1000;
        ssdb.data = Arc::new(self.encoded());
        ssdb
    }

    fn filename(&self) -> Cow<'_, str> {
        let end = self.filename.iter().position(|&b| b == 0).unwrap_or(self.filename.len());
        return String::from_utf8_lossy(&self.filename[..end]);
    }
}

impl Default for SpectrumHeader {
    fn default() -> Self {
        SpectrumHeader::new(SpectrumFileType::default(), "", 0, 0, 0)
    }
}

impl Into<DataPayload> for &SpectrumHeader {
    fn into(self) -> DataPayload 
    {
        let bytes = self.bytes();
        DataPayload::new(8, bytes.len() as u32, Arc::new(bytes))
    }
}

impl Payload for SpectrumHeader {
    fn bytes(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        self.write(&mut writer).unwrap();
        let mut encoded = writer.into_inner();
        encoded.truncate(17);
        return encoded;
    }

    fn clone_box(&self) -> Box<dyn Payload> {
        Box::new(self.clone())
    }

    fn flag_byte(&self) -> Option<u8> { Some(SpectrumFlag::SpectrumHeader as u8) }

    fn into_block_box(self: Box<Self>) -> Box<dyn Block> {
        Box::new((*self).into_standard_speed_data_block())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl fmt::Display for SpectrumHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SpectrumHeader: {:16} (type: {}; len: {}; p1: {:04x}; p2: {:04x})",
            self.filename(),
            self.file_type,
            self.data_length,
            self.parameter1,
            self.parameter2,
        )
    }
}

/// A block of ZX Spectrum data.
#[binrw]
#[br(little)]
#[br(import(payload_len: usize))]
#[br(stream = r, map_stream = |s| XorReader::new(s, SpectrumFlag::SpectrumData as u8))]
#[bw(stream = w, map_stream = |s| XorWriter::new(s, SpectrumFlag::SpectrumData as u8))]
#[derive(Debug, Clone, Hash)]
pub struct SpectrumData {
    #[br(count = payload_len)]
    pub data: Vec<u8>,
    #[br(temp, assert(checksum == r.xor() ^ checksum, "XOR mismatch: expected {:02X}, got {:02X}", r.xor() ^ checksum, checksum))]
    #[bw(calc(w.xor()))]
    pub checksum: u8,
}

impl SpectrumData {
    pub fn new(data: Vec<u8>) -> Self { SpectrumData { data } }

    pub fn encoded(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        SpectrumFlag::SpectrumData.write(&mut writer).unwrap();
        self.write_le(&mut writer).unwrap();
        return writer.into_inner();
    }

    pub fn into_standard_speed_data_block(&self) -> StandardSpeedDataBlock {
        let mut ssdb = StandardSpeedDataBlock::new();
        ssdb.pause = 2000;
        ssdb.data = Arc::new(self.encoded());
        ssdb
    }
}

impl Default for SpectrumData {
    fn default() -> Self { SpectrumData::new(Vec::new()) }
}

impl Payload for SpectrumData {
    fn bytes(&self) -> Vec<u8> { self.data.clone() }

    fn clone_box(&self) -> Box<dyn Payload> {
        Box::new(self.clone())
    }

    fn flag_byte(&self) -> Option<u8> { Some(SpectrumFlag::SpectrumData as u8) }

    fn into_block_box(self: Box<Self>) -> Box<dyn Block> {
        Box::new((*self).into_standard_speed_data_block())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl fmt::Display for SpectrumData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SpectrumData: {} bytes",
            self.data.len(),
        )
    }
}

/// Flag byte indicating whether a payload contains a Spectrum header or data.
#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug, Default, Eq, PartialEq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum SpectrumFlag {
    #[default]
    SpectrumHeader = 0x00,
    SpectrumData = 0xff,
}

/// Spectrum file type byte.
#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug, Default, Eq, PartialEq, Hash)]
pub enum SpectrumFileType {
    #[default]
    Program = 0x00,
    NumberArray = 0x01,
    CharacterArray = 0x02,
    CodeFile = 0x03,
}
